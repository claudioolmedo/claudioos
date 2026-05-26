use crate::bus::{Bus, BusFault, GPIOA_BASE, GPIOC_BASE, GPIOD_BASE, GPIO_BSHR_OFFSET, GPIO_OUT};
use crate::instruction::{decode, DecodeError, Instruction};
use crate::register::{Register, REGISTER_COUNT};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    GpioWrite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Event {
    pub cycle: u64,
    pub kind: EventKind,
    pub address: u32,
    pub value: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunLimit {
    pub max_cycles: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StopReason {
    Running,
    Ebreak,
    Ecall,
    MaxCycles,
    DecodeFault(DecodeError),
    BusFault(BusFault),
    Unsupported(Instruction),
}

pub struct Machine<B, const TRACE: usize> {
    bus: B,
    registers: [u32; REGISTER_COUNT],
    pc: u32,
    mepc: u32,
    mstatus: u32,
    mtvec: u32,
    cycles: u64,
    events: [Option<Event>; TRACE],
    event_count: usize,
}

impl<B: Bus, const TRACE: usize> Machine<B, TRACE> {
    pub const fn new(bus: B) -> Self {
        Self {
            bus,
            registers: [0; REGISTER_COUNT],
            pc: 0,
            mepc: 0,
            mstatus: 0,
            mtvec: 0,
            cycles: 0,
            events: [None; TRACE],
            event_count: 0,
        }
    }

    pub const fn pc(&self) -> u32 {
        self.pc
    }

    pub const fn cycles(&self) -> u64 {
        self.cycles
    }

    pub const fn bus(&self) -> &B {
        &self.bus
    }

    pub fn bus_mut(&mut self) -> &mut B {
        &mut self.bus
    }

    pub fn events(&self) -> impl Iterator<Item = Event> + '_ {
        self.events.iter().filter_map(|event| *event)
    }

    pub fn register(&self, register: Register) -> u32 {
        self.registers[register.index()]
    }

    pub fn set_register(&mut self, register: Register, value: u32) {
        if register != Register::ZERO {
            self.registers[register.index()] = value;
        }
    }

    pub fn run(&mut self, limit: RunLimit) -> StopReason {
        while self.cycles < limit.max_cycles {
            let reason = self.step();
            if reason != StopReason::Running {
                return reason;
            }
        }

        StopReason::MaxCycles
    }

    pub fn step(&mut self) -> StopReason {
        let low = match self.bus.read16(self.pc) {
            Ok(raw) => raw,
            Err(fault) => return StopReason::BusFault(fault),
        };

        if low & 0x03 != 0x03 {
            self.cycles = self.cycles.wrapping_add(1);
            return self.step_compressed(low);
        }

        let raw = match self.fetch_word(self.pc) {
            Ok(raw) => raw,
            Err(fault) => return StopReason::BusFault(fault),
        };

        self.cycles = self.cycles.wrapping_add(1);
        let next_pc = self.pc.wrapping_add(4);

        if raw == 0x3020_0073 {
            self.pc = self.mepc;
            return StopReason::Running;
        }

        if (raw & 0x7f) == 0x73 && ((raw >> 12) & 0x07) == 1 {
            let csr = (raw >> 20) & 0xfff;
            let rd = match Register::new(((raw >> 7) & 0x1f) as u8) {
                Ok(register) => register,
                Err(error) => return StopReason::DecodeFault(DecodeError::Register(error)),
            };
            let rs1 = match Register::new(((raw >> 15) & 0x1f) as u8) {
                Ok(register) => register,
                Err(error) => return StopReason::DecodeFault(DecodeError::Register(error)),
            };
            let old = self.read_csr(csr);
            self.write_csr(csr, self.read_register(rs1));
            self.write_register(rd, old);
            self.pc = next_pc;
            self.registers[Register::ZERO.index()] = 0;
            return StopReason::Running;
        }

        let instruction = match decode(raw) {
            Ok(instruction) => instruction,
            Err(fault) => return StopReason::DecodeFault(fault),
        };

        match instruction {
            Instruction::Lui { rd, imm } => {
                self.write_register(rd, imm);
                self.pc = next_pc;
            }
            Instruction::Auipc { rd, imm } => {
                self.write_register(rd, self.pc.wrapping_add(imm));
                self.pc = next_pc;
            }
            Instruction::Jal { rd, offset } => {
                self.write_register(rd, next_pc);
                self.pc = self.pc.wrapping_add(offset as u32);
            }
            Instruction::Jalr { rd, rs1, offset } => {
                let target = self.read_register(rs1).wrapping_add(offset as u32) & !1;
                self.write_register(rd, next_pc);
                self.pc = target;
            }
            Instruction::Branch {
                kind,
                rs1,
                rs2,
                offset,
            } => {
                let lhs = self.read_register(rs1);
                let rhs = self.read_register(rs2);
                self.pc = if branch_taken(kind, lhs, rhs) {
                    self.pc.wrapping_add(offset as u32)
                } else {
                    next_pc
                };
            }
            Instruction::Load {
                width,
                signed,
                rd,
                rs1,
                offset,
            } => {
                let address = self.read_register(rs1).wrapping_add(offset as u32);
                let value = match load_value(&mut self.bus, address, width, signed) {
                    Ok(value) => value,
                    Err(fault) => return StopReason::BusFault(fault),
                };
                self.write_register(rd, value);
                self.pc = next_pc;
            }
            Instruction::Store {
                width,
                rs1,
                rs2,
                offset,
            } => {
                let address = self.read_register(rs1).wrapping_add(offset as u32);
                let value = self.read_register(rs2);
                if is_gpio_event_address(address) {
                    self.record(Event {
                        cycle: self.cycles,
                        kind: EventKind::GpioWrite,
                        address,
                        value,
                    });
                }

                if let Err(fault) = store_value(&mut self.bus, address, width, value) {
                    return StopReason::BusFault(fault);
                }
                self.pc = next_pc;
            }
            Instruction::OpImm { kind, rd, rs1, imm } => {
                let lhs = self.read_register(rs1);
                let value = op_imm(kind, lhs, imm);
                self.write_register(rd, value);
                self.pc = next_pc;
            }
            Instruction::Op { kind, rd, rs1, rs2 } => {
                let lhs = self.read_register(rs1);
                let rhs = self.read_register(rs2);
                let value = op(kind, lhs, rhs);
                self.write_register(rd, value);
                self.pc = next_pc;
            }
            Instruction::Fence => {
                self.pc = next_pc;
            }
            Instruction::Ebreak => return StopReason::Ebreak,
            Instruction::Ecall => return StopReason::Ecall,
        }

        self.registers[Register::ZERO.index()] = 0;
        StopReason::Running
    }

    fn step_compressed(&mut self, raw: u16) -> StopReason {
        let opcode = raw & 0x03;
        let funct3 = (raw >> 13) & 0x07;
        let next_pc = self.pc.wrapping_add(2);

        match (opcode, funct3) {
            (0, 2) => {
                let rd = compressed_register((raw >> 2) & 0x07);
                let rs1 = compressed_register((raw >> 7) & 0x07);
                let offset = compressed_load_store_offset(raw);
                let address = self.read_register(rs1).wrapping_add(offset);
                let value = match self.bus.read32_mut(address) {
                    Ok(value) => value,
                    Err(fault) => return StopReason::BusFault(fault),
                };
                self.write_register(rd, value);
                self.pc = next_pc;
            }
            (0, 6) => {
                let rs2 = compressed_register((raw >> 2) & 0x07);
                let rs1 = compressed_register((raw >> 7) & 0x07);
                let offset = compressed_load_store_offset(raw);
                let address = self.read_register(rs1).wrapping_add(offset);
                let value = self.read_register(rs2);
                if is_gpio_event_address(address) {
                    self.record(Event {
                        cycle: self.cycles,
                        kind: EventKind::GpioWrite,
                        address,
                        value,
                    });
                }
                if let Err(fault) = self.bus.write32(address, value) {
                    return StopReason::BusFault(fault);
                }
                self.pc = next_pc;
            }
            (1, 0) => {
                let rd = raw_register((raw >> 7) & 0x1f);
                let imm = sign_extend_u32(
                    (((raw >> 12) & 1) << 5) as u32 | ((raw >> 2) & 0x1f) as u32,
                    6,
                );
                let value = self.read_register(rd).wrapping_add(imm as u32);
                self.write_register(rd, value);
                self.pc = next_pc;
            }
            (1, 1) => {
                self.write_register(Register::RETURN_ADDRESS, next_pc);
                self.pc = self.pc.wrapping_add(compressed_jump_offset(raw) as u32);
            }
            (1, 2) => {
                let rd = raw_register((raw >> 7) & 0x1f);
                let imm = sign_extend_u32(
                    (((raw >> 12) & 1) << 5) as u32 | ((raw >> 2) & 0x1f) as u32,
                    6,
                );
                self.write_register(rd, imm as u32);
                self.pc = next_pc;
            }
            (1, 3) => {
                let rd = raw_register((raw >> 7) & 0x1f);
                if rd == Register::STACK_POINTER {
                    let imm = compressed_addi16sp_offset(raw);
                    let value = self.read_register(rd).wrapping_add(imm as u32);
                    self.write_register(rd, value);
                } else {
                    let imm = sign_extend_u32(
                        (((raw >> 12) & 1) << 5) as u32 | ((raw >> 2) & 0x1f) as u32,
                        6,
                    );
                    self.write_register(rd, (imm as u32) << 12);
                }
                self.pc = next_pc;
            }
            (1, 4) => {
                let op = (raw >> 10) & 0x03;
                let rd = compressed_register((raw >> 7) & 0x07);
                if op == 0 {
                    let shamt = (((raw >> 12) & 1) << 5) | ((raw >> 2) & 0x1f);
                    let value = self.read_register(rd) >> shamt;
                    self.write_register(rd, value);
                } else if op == 1 {
                    let shamt = (((raw >> 12) & 1) << 5) | ((raw >> 2) & 0x1f);
                    let value = ((self.read_register(rd) as i32) >> shamt) as u32;
                    self.write_register(rd, value);
                } else if op == 2 {
                    let imm = sign_extend_u32(
                        (((raw >> 12) & 1) << 5) as u32 | ((raw >> 2) & 0x1f) as u32,
                        6,
                    );
                    let value = self.read_register(rd) & imm as u32;
                    self.write_register(rd, value);
                } else if op == 3 {
                    let rs2 = compressed_register((raw >> 2) & 0x07);
                    let value = match ((raw >> 12) & 1, (raw >> 5) & 0x03) {
                        (0, 0) => self.read_register(rd).wrapping_sub(self.read_register(rs2)),
                        (0, 2) => self.read_register(rd) | self.read_register(rs2),
                        (0, 3) => self.read_register(rd) & self.read_register(rs2),
                        _ => {
                            return StopReason::DecodeFault(DecodeError::UnknownOpcode(
                                (raw & 0xff) as u8,
                            ))
                        }
                    };
                    self.write_register(rd, value);
                } else {
                    return StopReason::DecodeFault(DecodeError::UnknownOpcode((raw & 0xff) as u8));
                }
                self.pc = next_pc;
            }
            (1, 5) => {
                self.pc = self.pc.wrapping_add(compressed_jump_offset(raw) as u32);
            }
            (1, 6) | (1, 7) => {
                let rs1 = compressed_register((raw >> 7) & 0x07);
                let offset = compressed_branch_offset(raw);
                let value = self.read_register(rs1);
                let taken = if funct3 == 6 { value == 0 } else { value != 0 };
                self.pc = if taken {
                    self.pc.wrapping_add(offset as u32)
                } else {
                    next_pc
                };
            }
            (2, 0) => {
                let rd = raw_register((raw >> 7) & 0x1f);
                let shamt = (((raw >> 12) & 1) << 5) | ((raw >> 2) & 0x1f);
                self.write_register(rd, self.read_register(rd) << shamt);
                self.pc = next_pc;
            }
            (2, 2) => {
                let rd = raw_register((raw >> 7) & 0x1f);
                let offset = (((raw >> 12) & 1) << 5)
                    | (((raw >> 4) & 0x07) << 2)
                    | (((raw >> 2) & 0x03) << 6);
                let address = self
                    .read_register(Register::STACK_POINTER)
                    .wrapping_add(offset as u32);
                let value = match self.bus.read32_mut(address) {
                    Ok(value) => value,
                    Err(fault) => return StopReason::BusFault(fault),
                };
                self.write_register(rd, value);
                self.pc = next_pc;
            }
            (2, 4) => {
                let rd = raw_register((raw >> 7) & 0x1f);
                let rs2 = raw_register((raw >> 2) & 0x1f);
                if ((raw >> 12) & 1) == 0 {
                    if rs2 == Register::ZERO {
                        if rd == Register::ZERO {
                            return StopReason::DecodeFault(DecodeError::UnknownOpcode(
                                (raw & 0xff) as u8,
                            ));
                        }
                        self.pc = self.read_register(rd) & !1;
                    } else {
                        self.write_register(rd, self.read_register(rs2));
                        self.pc = next_pc;
                    }
                } else if rs2 == Register::ZERO {
                    if rd == Register::ZERO {
                        return StopReason::Ebreak;
                    }
                    self.write_register(Register::RETURN_ADDRESS, next_pc);
                    self.pc = self.read_register(rd) & !1;
                } else {
                    let value = self.read_register(rd).wrapping_add(self.read_register(rs2));
                    self.write_register(rd, value);
                    self.pc = next_pc;
                }
            }
            (2, 6) => {
                let rs2 = raw_register((raw >> 2) & 0x1f);
                let offset = (((raw >> 7) & 0x3) << 6) | (((raw >> 9) & 0x0f) << 2);
                let address = self
                    .read_register(Register::STACK_POINTER)
                    .wrapping_add(offset as u32);
                let value = self.read_register(rs2);
                if let Err(fault) = self.bus.write32(address, value) {
                    return StopReason::BusFault(fault);
                }
                self.pc = next_pc;
            }
            _ => return StopReason::DecodeFault(DecodeError::UnknownOpcode((raw & 0xff) as u8)),
        }

        self.registers[Register::ZERO.index()] = 0;
        StopReason::Running
    }

    fn read_register(&self, register: Register) -> u32 {
        self.registers[register.index()]
    }

    fn fetch_word(&mut self, address: u32) -> Result<u32, BusFault> {
        if address & 0x01 != 0 {
            return Err(BusFault::Unaligned { address });
        }

        let b0 = self.bus.read8(address)? as u32;
        let b1 = self.bus.read8(address.wrapping_add(1))? as u32;
        let b2 = self.bus.read8(address.wrapping_add(2))? as u32;
        let b3 = self.bus.read8(address.wrapping_add(3))? as u32;
        Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }

    fn write_register(&mut self, register: Register, value: u32) {
        if register != Register::ZERO {
            self.registers[register.index()] = value;
        }
    }

    fn read_csr(&self, csr: u32) -> u32 {
        match csr {
            0x300 => self.mstatus,
            0x305 => self.mtvec,
            0x341 => self.mepc,
            _ => 0,
        }
    }

    fn write_csr(&mut self, csr: u32, value: u32) {
        match csr {
            0x300 => self.mstatus = value,
            0x305 => self.mtvec = value,
            0x341 => self.mepc = value,
            _ => {}
        }
    }

    fn record(&mut self, event: Event) {
        if self.event_count < TRACE {
            self.events[self.event_count] = Some(event);
            self.event_count += 1;
        }
    }
}

fn raw_register(index: u16) -> Register {
    Register::new(index as u8).unwrap_or(Register::ZERO)
}

fn compressed_register(index: u16) -> Register {
    raw_register(index + 8)
}

fn compressed_load_store_offset(raw: u16) -> u32 {
    ((((raw >> 6) & 0x01) << 2) | (((raw >> 10) & 0x07) << 3) | (((raw >> 5) & 0x01) << 6)) as u32
}

fn compressed_jump_offset(raw: u16) -> i32 {
    let value = (((raw >> 12) & 0x01) << 11)
        | (((raw >> 11) & 0x01) << 4)
        | (((raw >> 9) & 0x03) << 8)
        | (((raw >> 8) & 0x01) << 10)
        | (((raw >> 7) & 0x01) << 6)
        | (((raw >> 6) & 0x01) << 7)
        | (((raw >> 3) & 0x07) << 1)
        | (((raw >> 2) & 0x01) << 5);
    sign_extend_u32(value as u32, 12)
}

fn compressed_branch_offset(raw: u16) -> i32 {
    let value = (((raw >> 12) & 0x01) << 8)
        | (((raw >> 10) & 0x03) << 3)
        | (((raw >> 5) & 0x03) << 6)
        | (((raw >> 3) & 0x03) << 1)
        | (((raw >> 2) & 0x01) << 5);
    sign_extend_u32(value as u32, 9)
}

fn compressed_addi16sp_offset(raw: u16) -> i32 {
    let value = (((raw >> 12) & 0x01) << 9)
        | (((raw >> 6) & 0x01) << 4)
        | (((raw >> 5) & 0x01) << 6)
        | (((raw >> 3) & 0x03) << 7)
        | (((raw >> 2) & 0x01) << 5);
    sign_extend_u32(value as u32, 10)
}

fn sign_extend_u32(value: u32, bits: u8) -> i32 {
    let shift = 32 - bits;
    ((value << shift) as i32) >> shift
}

fn is_gpio_event_address(address: u32) -> bool {
    address == GPIO_OUT
        || address == GPIOA_BASE + GPIO_BSHR_OFFSET
        || address == GPIOC_BASE + GPIO_BSHR_OFFSET
        || address == GPIOD_BASE + GPIO_BSHR_OFFSET
}

fn load_value<B: Bus>(bus: &mut B, address: u32, width: u8, signed: bool) -> Result<u32, BusFault> {
    match width {
        1 => {
            let value = bus.read8(address)? as u32;
            Ok(if signed {
                sign_extend_u32(value, 8) as u32
            } else {
                value
            })
        }
        2 => {
            let value = bus.read16(address)? as u32;
            Ok(if signed {
                sign_extend_u32(value, 16) as u32
            } else {
                value
            })
        }
        _ => bus.read32_mut(address),
    }
}

fn store_value<B: Bus>(bus: &mut B, address: u32, width: u8, value: u32) -> Result<(), BusFault> {
    match width {
        1 => bus.write8(address, value as u8),
        2 => bus.write16(address, value as u16),
        _ => bus.write32(address, value),
    }
}

fn branch_taken(kind: u8, lhs: u32, rhs: u32) -> bool {
    match kind {
        0 => lhs == rhs,
        1 => lhs != rhs,
        4 => (lhs as i32) < (rhs as i32),
        5 => (lhs as i32) >= (rhs as i32),
        6 => lhs < rhs,
        7 => lhs >= rhs,
        _ => false,
    }
}

fn op_imm(kind: u8, lhs: u32, imm: i32) -> u32 {
    let rhs = imm as u32;
    match kind {
        0 => lhs.wrapping_add(rhs),
        1 => lhs << (rhs & 0x1f),
        2 => ((lhs as i32) < imm) as u32,
        3 => (lhs < rhs) as u32,
        4 => lhs ^ rhs,
        5 => lhs >> (rhs & 0x1f),
        6 => lhs | rhs,
        7 => lhs & rhs,
        _ => 0,
    }
}

fn op(kind: u8, lhs: u32, rhs: u32) -> u32 {
    match kind {
        0 => lhs.wrapping_add(rhs),
        1 => lhs << (rhs & 0x1f),
        2 => ((lhs as i32) < (rhs as i32)) as u32,
        3 => (lhs < rhs) as u32,
        4 => lhs ^ rhs,
        5 => lhs >> (rhs & 0x1f),
        6 => lhs | rhs,
        7 => lhs & rhs,
        8 => lhs.wrapping_sub(rhs),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::RamBus;

    const LUI_X1_GPIO: u32 = 0x4001_10b7;
    const ADDI_X2_ONE: u32 = 0x0010_0113;
    const SW_X2_GPIO_OUT: u32 = 0x0020_a623;
    const SW_ZERO_GPIO_OUT: u32 = 0x0000_a623;
    const EBREAK: u32 = 0x0010_0073;

    #[test]
    fn runs_blink_like_gpio_program() {
        let mut bus = RamBus::<8>::new();
        bus.load_word(0, LUI_X1_GPIO).unwrap();
        bus.load_word(1, ADDI_X2_ONE).unwrap();
        bus.load_word(2, SW_X2_GPIO_OUT).unwrap();
        bus.load_word(3, SW_ZERO_GPIO_OUT).unwrap();
        bus.load_word(4, EBREAK).unwrap();

        let mut machine = Machine::<_, 8>::new(bus);
        let reason = machine.run(RunLimit { max_cycles: 16 });

        assert_eq!(reason, StopReason::Ebreak);
        assert_eq!(machine.bus().gpio_out(), 0);

        let events: [Event; 2] = {
            let mut iter = machine.events();
            [iter.next().unwrap(), iter.next().unwrap()]
        };

        assert_eq!(events[0].value, 1);
        assert_eq!(events[1].value, 0);
    }
}
