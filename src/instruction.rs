use crate::register::{Register, RegisterError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Opcode {
    Lui,
    Auipc,
    Jal,
    Jalr,
    Branch,
    Load,
    Store,
    OpImm,
    Op,
    Fence,
    System,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instruction {
    Lui {
        rd: Register,
        imm: u32,
    },
    Auipc {
        rd: Register,
        imm: u32,
    },
    Jal {
        rd: Register,
        offset: i32,
    },
    Jalr {
        rd: Register,
        rs1: Register,
        offset: i32,
    },
    Branch {
        kind: u8,
        rs1: Register,
        rs2: Register,
        offset: i32,
    },
    Load {
        width: u8,
        signed: bool,
        rd: Register,
        rs1: Register,
        offset: i32,
    },
    Store {
        width: u8,
        rs1: Register,
        rs2: Register,
        offset: i32,
    },
    OpImm {
        kind: u8,
        rd: Register,
        rs1: Register,
        imm: i32,
    },
    Op {
        kind: u8,
        rd: Register,
        rs1: Register,
        rs2: Register,
    },
    Fence,
    Ecall,
    Ebreak,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    UnknownOpcode(u8),
    UnknownSystem(u32),
    Register(RegisterError),
}

impl From<RegisterError> for DecodeError {
    fn from(value: RegisterError) -> Self {
        Self::Register(value)
    }
}

pub fn decode(raw: u32) -> Result<Instruction, DecodeError> {
    let opcode = raw & 0x7f;
    let funct3 = ((raw >> 12) & 0x07) as u8;

    match opcode {
        0x37 => Ok(Instruction::Lui {
            rd: rd(raw)?,
            imm: raw & 0xffff_f000,
        }),
        0x17 => Ok(Instruction::Auipc {
            rd: rd(raw)?,
            imm: raw & 0xffff_f000,
        }),
        0x6f => Ok(Instruction::Jal {
            rd: rd(raw)?,
            offset: imm_j(raw),
        }),
        0x67 => Ok(Instruction::Jalr {
            rd: rd(raw)?,
            rs1: rs1(raw)?,
            offset: imm_i(raw),
        }),
        0x63 => Ok(Instruction::Branch {
            kind: funct3,
            rs1: rs1(raw)?,
            rs2: rs2(raw)?,
            offset: imm_b(raw),
        }),
        0x03 => Ok(Instruction::Load {
            width: load_width(funct3),
            signed: funct3 < 4,
            rd: rd(raw)?,
            rs1: rs1(raw)?,
            offset: imm_i(raw),
        }),
        0x23 => Ok(Instruction::Store {
            width: store_width(funct3),
            rs1: rs1(raw)?,
            rs2: rs2(raw)?,
            offset: imm_s(raw),
        }),
        0x13 => Ok(Instruction::OpImm {
            kind: funct3,
            rd: rd(raw)?,
            rs1: rs1(raw)?,
            imm: imm_i(raw),
        }),
        0x33 => Ok(Instruction::Op {
            kind: op_kind(raw),
            rd: rd(raw)?,
            rs1: rs1(raw)?,
            rs2: rs2(raw)?,
        }),
        0x0f => Ok(Instruction::Fence),
        0x73 => match raw {
            0x0000_0073 => Ok(Instruction::Ecall),
            0x0010_0073 => Ok(Instruction::Ebreak),
            _ => Err(DecodeError::UnknownSystem(raw)),
        },
        other => Err(DecodeError::UnknownOpcode(other as u8)),
    }
}

fn rd(raw: u32) -> Result<Register, DecodeError> {
    reg((raw >> 7) & 0x1f)
}

fn rs1(raw: u32) -> Result<Register, DecodeError> {
    reg((raw >> 15) & 0x1f)
}

fn rs2(raw: u32) -> Result<Register, DecodeError> {
    reg((raw >> 20) & 0x1f)
}

fn reg(raw: u32) -> Result<Register, DecodeError> {
    Ok(Register::new(raw as u8)?)
}

fn load_width(funct3: u8) -> u8 {
    match funct3 & 0x03 {
        0 => 1,
        1 => 2,
        _ => 4,
    }
}

fn store_width(funct3: u8) -> u8 {
    match funct3 {
        0 => 1,
        1 => 2,
        _ => 4,
    }
}

fn op_kind(raw: u32) -> u8 {
    let funct3 = ((raw >> 12) & 0x07) as u8;
    let funct7 = ((raw >> 25) & 0x7f) as u8;
    funct3 | ((funct7 >> 5) << 3)
}

fn sign_extend(value: u32, bits: u8) -> i32 {
    let shift = 32 - bits;
    ((value << shift) as i32) >> shift
}

fn imm_i(raw: u32) -> i32 {
    sign_extend(raw >> 20, 12)
}

fn imm_s(raw: u32) -> i32 {
    let value = ((raw >> 25) << 5) | ((raw >> 7) & 0x1f);
    sign_extend(value, 12)
}

fn imm_b(raw: u32) -> i32 {
    let value = (((raw >> 31) & 0x01) << 12)
        | (((raw >> 7) & 0x01) << 11)
        | (((raw >> 25) & 0x3f) << 5)
        | (((raw >> 8) & 0x0f) << 1);
    sign_extend(value, 13)
}

fn imm_j(raw: u32) -> i32 {
    let value = (((raw >> 31) & 0x01) << 20)
        | (((raw >> 12) & 0xff) << 12)
        | (((raw >> 20) & 0x01) << 11)
        | (((raw >> 21) & 0x03ff) << 1);
    sign_extend(value, 21)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_addi() {
        let instruction = decode(0x0010_8093).unwrap();
        assert_eq!(
            instruction,
            Instruction::OpImm {
                kind: 0,
                rd: Register::new(1).unwrap(),
                rs1: Register::new(1).unwrap(),
                imm: 1,
            }
        );
    }

    #[test]
    fn decodes_ebreak() {
        assert_eq!(decode(0x0010_0073).unwrap(), Instruction::Ebreak);
    }

    #[test]
    fn rejects_registers_outside_rv32e() {
        assert_eq!(
            decode(0x0010_8f93),
            Err(DecodeError::Register(RegisterError::NotRv32e(31)))
        );
    }
}
