pub const GPIO_BASE: u32 = 0x4001_1000;
pub const GPIO_OUT: u32 = GPIO_BASE + 0x0c;
pub const GPIOA_BASE: u32 = 0x4001_0800;
pub const GPIOC_BASE: u32 = 0x4001_1000;
pub const GPIOD_BASE: u32 = 0x4001_1400;
pub const GPIO_OUT_OFFSET: u32 = 0x0c;
pub const GPIO_BSHR_OFFSET: u32 = 0x10;
pub const GPIO_BCR_OFFSET: u32 = 0x14;

const TIM2_BASE: u32 = 0x4000_0000;
const I2C1_BASE: u32 = 0x4000_5400;
const AFIO_BASE: u32 = 0x4001_0000;
const EXTI_BASE: u32 = 0x4001_0400;
const ADC1_BASE: u32 = 0x4001_2400;
const TIM1_BASE: u32 = 0x4001_2c00;
const SPI1_BASE: u32 = 0x4001_3000;
const USART1_BASE: u32 = 0x4001_3800;
const DMA1_BASE: u32 = 0x4002_0000;
const PERIPHERAL_REGISTER_SLOTS: usize = 128;
const SERIAL_BUFFER_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BusFault {
    Unaligned { address: u32 },
    OutOfRange { address: u32 },
}

pub trait Bus {
    fn read8(&mut self, address: u32) -> Result<u8, BusFault>;
    fn write8(&mut self, address: u32, value: u8) -> Result<(), BusFault>;

    fn read16(&mut self, address: u32) -> Result<u16, BusFault> {
        if address & 0x01 != 0 {
            return Err(BusFault::Unaligned { address });
        }

        let lo = self.read8(address)? as u16;
        let hi = self.read8(address.wrapping_add(1))? as u16;
        Ok(lo | (hi << 8))
    }

    fn read32(&self, address: u32) -> Result<u32, BusFault>;

    fn write16(&mut self, address: u32, value: u16) -> Result<(), BusFault> {
        if address & 0x01 != 0 {
            return Err(BusFault::Unaligned { address });
        }

        self.write8(address, value as u8)?;
        self.write8(address.wrapping_add(1), (value >> 8) as u8)
    }

    fn read32_mut(&mut self, address: u32) -> Result<u32, BusFault> {
        if address & 0x03 != 0 {
            return Err(BusFault::Unaligned { address });
        }

        let b0 = self.read8(address)? as u32;
        let b1 = self.read8(address.wrapping_add(1))? as u32;
        let b2 = self.read8(address.wrapping_add(2))? as u32;
        let b3 = self.read8(address.wrapping_add(3))? as u32;
        Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }

    fn write32(&mut self, address: u32, value: u32) -> Result<(), BusFault>;
}

#[derive(Clone, Debug)]
pub struct RamBus<const WORDS: usize> {
    words: [u32; WORDS],
    gpio_out: u32,
}

impl<const WORDS: usize> RamBus<WORDS> {
    pub const fn new() -> Self {
        Self {
            words: [0; WORDS],
            gpio_out: 0,
        }
    }

    pub fn load_word(&mut self, index: usize, word: u32) -> Result<(), BusFault> {
        if index < WORDS {
            self.words[index] = word;
            Ok(())
        } else {
            Err(BusFault::OutOfRange {
                address: (index as u32).wrapping_mul(4),
            })
        }
    }

    pub const fn gpio_out(&self) -> u32 {
        self.gpio_out
    }

    fn memory_index(address: u32) -> Result<usize, BusFault> {
        if address & 0x03 != 0 {
            return Err(BusFault::Unaligned { address });
        }

        Ok((address >> 2) as usize)
    }
}

impl<const WORDS: usize> Default for RamBus<WORDS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const WORDS: usize> Bus for RamBus<WORDS> {
    fn read8(&mut self, address: u32) -> Result<u8, BusFault> {
        let aligned = address & !0x03;
        let shift = (address & 0x03) * 8;
        Ok(((self.read32(aligned)? >> shift) & 0xff) as u8)
    }

    fn write8(&mut self, address: u32, value: u8) -> Result<(), BusFault> {
        let aligned = address & !0x03;
        let shift = (address & 0x03) * 8;
        let mask = !(0xff << shift);
        let current = self.read32(aligned)?;
        self.write32(aligned, (current & mask) | ((value as u32) << shift))
    }

    fn read32(&self, address: u32) -> Result<u32, BusFault> {
        if address == GPIO_OUT {
            return Ok(self.gpio_out);
        }

        let index = Self::memory_index(address)?;
        self.words
            .get(index)
            .copied()
            .ok_or(BusFault::OutOfRange { address })
    }

    fn write32(&mut self, address: u32, value: u32) -> Result<(), BusFault> {
        if address == GPIO_OUT {
            self.gpio_out = value;
            return Ok(());
        }

        let index = Self::memory_index(address)?;
        let word = self
            .words
            .get_mut(index)
            .ok_or(BusFault::OutOfRange { address })?;
        *word = value;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct BoardBus<const FLASH: usize, const RAM: usize> {
    flash: [u8; FLASH],
    ram: [u8; RAM],
    gpio_a: GpioPort,
    gpio_c: GpioPort,
    gpio_d: GpioPort,
    rcc_ctlr: u32,
    rcc_cfgr0: u32,
    rcc_ahbpcenr: u32,
    rcc_apb2pcenr: u32,
    rcc_apb1pcenr: u32,
    flash_actlr: u32,
    systick_ctlr: u32,
    systick_count: u32,
    mmio: [(u32, u32); PERIPHERAL_REGISTER_SLOTS],
    mmio_len: usize,
    usart1_tx: [u8; SERIAL_BUFFER_BYTES],
    usart1_tx_len: usize,
    spi1_tx: [u8; SERIAL_BUFFER_BYTES],
    spi1_tx_len: usize,
    i2c1_tx: [u8; SERIAL_BUFFER_BYTES],
    i2c1_tx_len: usize,
    dma_transfer_count: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GpioPort {
    pub cfglr: u32,
    pub out: u32,
    pub bshr_writes: u32,
}

impl<const FLASH: usize, const RAM: usize> BoardBus<FLASH, RAM> {
    pub const RAM_BASE: u32 = 0x2000_0000;
    pub const SYSTICK_BASE: u32 = 0xe000_f000;
    pub const RCC_BASE: u32 = 0x4002_1000;
    pub const FLASH_BASE: u32 = 0x4002_2000;

    pub const fn new() -> Self {
        Self {
            flash: [0; FLASH],
            ram: [0; RAM],
            gpio_a: GpioPort {
                cfglr: 0,
                out: 0,
                bshr_writes: 0,
            },
            gpio_c: GpioPort {
                cfglr: 0,
                out: 0,
                bshr_writes: 0,
            },
            gpio_d: GpioPort {
                cfglr: 0,
                out: 0,
                bshr_writes: 0,
            },
            rcc_ctlr: 0x0200_0000,
            rcc_cfgr0: 0x0000_0008,
            rcc_ahbpcenr: 0,
            rcc_apb2pcenr: 0,
            rcc_apb1pcenr: 0,
            flash_actlr: 0,
            systick_ctlr: 0,
            systick_count: 0,
            mmio: [(0, 0); PERIPHERAL_REGISTER_SLOTS],
            mmio_len: 0,
            usart1_tx: [0; SERIAL_BUFFER_BYTES],
            usart1_tx_len: 0,
            spi1_tx: [0; SERIAL_BUFFER_BYTES],
            spi1_tx_len: 0,
            i2c1_tx: [0; SERIAL_BUFFER_BYTES],
            i2c1_tx_len: 0,
            dma_transfer_count: 0,
        }
    }

    pub fn load_flash(&mut self, bytes: &[u8]) -> Result<(), BusFault> {
        if bytes.len() > FLASH {
            return Err(BusFault::OutOfRange {
                address: bytes.len() as u32,
            });
        }

        self.flash[..bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    pub const fn gpio_a(&self) -> GpioPort {
        self.gpio_a
    }

    pub const fn gpio_c(&self) -> GpioPort {
        self.gpio_c
    }

    pub const fn gpio_d(&self) -> GpioPort {
        self.gpio_d
    }

    pub fn usart1_tx(&self) -> &[u8] {
        &self.usart1_tx[..self.usart1_tx_len]
    }

    pub fn spi1_tx(&self) -> &[u8] {
        &self.spi1_tx[..self.spi1_tx_len]
    }

    pub fn i2c1_tx(&self) -> &[u8] {
        &self.i2c1_tx[..self.i2c1_tx_len]
    }

    pub const fn dma_transfer_count(&self) -> u32 {
        self.dma_transfer_count
    }

    fn read_memory(&self, address: u32) -> Result<u8, BusFault> {
        if let Some(index) = address.checked_sub(Self::RAM_BASE) {
            return self
                .ram
                .get(index as usize)
                .copied()
                .ok_or(BusFault::OutOfRange { address });
        }

        self.flash
            .get(address as usize)
            .copied()
            .ok_or(BusFault::OutOfRange { address })
    }

    fn write_memory(&mut self, address: u32, value: u8) -> Result<(), BusFault> {
        if let Some(index) = address.checked_sub(Self::RAM_BASE) {
            let byte = self
                .ram
                .get_mut(index as usize)
                .ok_or(BusFault::OutOfRange { address })?;
            *byte = value;
            return Ok(());
        }

        Err(BusFault::OutOfRange { address })
    }

    fn read_mmio32(&mut self, address: u32) -> Option<u32> {
        if let Some(value) = read_gpio(&self.gpio_a, GPIOA_BASE, address) {
            return Some(value);
        }
        if let Some(value) = read_gpio(&self.gpio_c, GPIOC_BASE, address) {
            return Some(value);
        }
        if let Some(value) = read_gpio(&self.gpio_d, GPIOD_BASE, address) {
            return Some(value);
        }

        match address {
            0xe000_f000 => Some(self.systick_ctlr),
            0xe000_f008 => {
                self.systick_count = self.systick_count.wrapping_add(50_000);
                Some(self.systick_count)
            }
            0x4002_1000 => Some(self.rcc_ctlr | 0x0300_0003),
            0x4002_1004 => Some((self.rcc_cfgr0 & !0x0c) | 0x08),
            0x4002_1008 => Some(0),
            0x4002_1014 => Some(self.rcc_ahbpcenr),
            0x4002_1018 => Some(self.rcc_apb2pcenr),
            0x4002_101c => Some(self.rcc_apb1pcenr),
            0x4002_2000 => Some(self.flash_actlr),
            address if address == USART1_BASE => Some(0x00c0),
            address if address == USART1_BASE + 0x04 => Some(0),
            address if address == SPI1_BASE + 0x08 => Some(0x0002),
            address if address == SPI1_BASE + 0x0c => Some(0xff),
            address if address == I2C1_BASE + 0x14 => Some(0x0080),
            address if address == I2C1_BASE + 0x18 => Some(0),
            address if address == I2C1_BASE + 0x10 => Some(0xff),
            address if address == ADC1_BASE => Some(0x0002),
            address if address == ADC1_BASE + 0x4c => Some(2048),
            address if address == TIM1_BASE + 0x10 || address == TIM2_BASE + 0x10 => Some(1),
            address if is_peripheral_mmio(address) => {
                Some(self.read_stored_mmio(address).unwrap_or(0))
            }
            _ => None,
        }
    }

    fn write_mmio32(&mut self, address: u32, value: u32) -> bool {
        if write_gpio(&mut self.gpio_a, GPIOA_BASE, address, value)
            || write_gpio(&mut self.gpio_c, GPIOC_BASE, address, value)
            || write_gpio(&mut self.gpio_d, GPIOD_BASE, address, value)
        {
            return true;
        }

        match address {
            0xe000_f000 => self.systick_ctlr = value,
            0x4002_1000 => self.rcc_ctlr = value | 0x0300_0003,
            0x4002_1004 => self.rcc_cfgr0 = (value & !0x0c) | 0x08,
            0x4002_1008 => {}
            0x4002_1014 => self.rcc_ahbpcenr = value,
            0x4002_1018 => self.rcc_apb2pcenr = value,
            0x4002_101c => self.rcc_apb1pcenr = value,
            0x4002_2000 => self.flash_actlr = value,
            address if address == USART1_BASE + 0x04 => {
                push_byte(&mut self.usart1_tx, &mut self.usart1_tx_len, value as u8);
                self.write_stored_mmio(address, value);
            }
            address if address == SPI1_BASE + 0x0c => {
                push_byte(&mut self.spi1_tx, &mut self.spi1_tx_len, value as u8);
                self.write_stored_mmio(address, value);
            }
            address if address == I2C1_BASE + 0x10 => {
                push_byte(&mut self.i2c1_tx, &mut self.i2c1_tx_len, value as u8);
                self.write_stored_mmio(address, value);
            }
            address if is_dma_channel_control(address) => {
                self.write_stored_mmio(address, value);
                if value & 0x01 != 0 {
                    self.perform_dma_transfer(address);
                }
            }
            address if is_peripheral_mmio(address) => self.write_stored_mmio(address, value),
            _ => return false,
        }

        true
    }

    fn read_stored_mmio(&self, address: u32) -> Option<u32> {
        self.mmio[..self.mmio_len]
            .iter()
            .find_map(|(stored_address, value)| (*stored_address == address).then_some(*value))
    }

    fn write_stored_mmio(&mut self, address: u32, value: u32) {
        if let Some((_, stored)) = self.mmio[..self.mmio_len]
            .iter_mut()
            .find(|(stored_address, _)| *stored_address == address)
        {
            *stored = value;
            return;
        }

        if self.mmio_len < self.mmio.len() {
            self.mmio[self.mmio_len] = (address, value);
            self.mmio_len += 1;
        }
    }

    fn perform_dma_transfer(&mut self, control_address: u32) {
        let channel_base = control_address;
        let count = self
            .read_stored_mmio(channel_base + 0x04)
            .unwrap_or(0)
            .min(1024);
        let peripheral = self.read_stored_mmio(channel_base + 0x08).unwrap_or(0);
        let memory = self.read_stored_mmio(channel_base + 0x0c).unwrap_or(0);
        let control = self.read_stored_mmio(channel_base).unwrap_or(0);
        let mem2mem = control & (1 << 14) != 0;
        let dir_mem_to_peripheral = control & (1 << 4) != 0;
        let pinc = control & (1 << 6) != 0;
        let minc = control & (1 << 7) != 0;
        let psize = transfer_width((control >> 8) & 0x03);
        let msize = transfer_width((control >> 10) & 0x03);
        let width = psize.max(msize);

        for index in 0..count {
            let step = index.wrapping_mul(width);
            let (src, dst) = if mem2mem {
                (
                    peripheral.wrapping_add(step),
                    memory.wrapping_add(if minc { step } else { 0 }),
                )
            } else if dir_mem_to_peripheral {
                (
                    memory.wrapping_add(if minc { step } else { 0 }),
                    peripheral.wrapping_add(if pinc { step } else { 0 }),
                )
            } else {
                (
                    peripheral.wrapping_add(if pinc { step } else { 0 }),
                    memory.wrapping_add(if minc { step } else { 0 }),
                )
            };

            for byte in 0..width {
                let value = match self.read_memory(src.wrapping_add(byte)) {
                    Ok(value) => value,
                    Err(_) => break,
                };
                if self.write_memory(dst.wrapping_add(byte), value).is_err() {
                    break;
                }
            }
        }

        self.dma_transfer_count = self.dma_transfer_count.wrapping_add(1);
        self.write_stored_mmio(
            DMA1_BASE,
            self.read_stored_mmio(DMA1_BASE).unwrap_or(0) | 0x02,
        );
    }
}

impl<const FLASH: usize, const RAM: usize> Default for BoardBus<FLASH, RAM> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const FLASH: usize, const RAM: usize> Bus for BoardBus<FLASH, RAM> {
    fn read8(&mut self, address: u32) -> Result<u8, BusFault> {
        let aligned = address & !0x03;
        if is_peripheral_mmio(aligned) {
            let shift = (address & 0x03) * 8;
            return Ok(((self.read_mmio32(aligned).unwrap_or(0) >> shift) & 0xff) as u8);
        }

        self.read_memory(address)
    }

    fn write8(&mut self, address: u32, value: u8) -> Result<(), BusFault> {
        let aligned = address & !0x03;
        if is_peripheral_mmio(aligned) {
            let shift = (address & 0x03) * 8;
            let mask = !(0xff << shift);
            let current = self.read_stored_mmio(aligned).unwrap_or(0);
            self.write_mmio32(aligned, (current & mask) | ((value as u32) << shift));
            return Ok(());
        }

        self.write_memory(address, value)
    }

    fn read32(&self, address: u32) -> Result<u32, BusFault> {
        if address & 0x03 != 0 {
            return Err(BusFault::Unaligned { address });
        }

        if let Some(index) = address.checked_sub(Self::RAM_BASE) {
            let index = index as usize;
            let bytes = self
                .ram
                .get(index..index + 4)
                .ok_or(BusFault::OutOfRange { address })?;
            return Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
        }

        let index = address as usize;
        let bytes = self
            .flash
            .get(index..index + 4)
            .ok_or(BusFault::OutOfRange { address })?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read32_mut(&mut self, address: u32) -> Result<u32, BusFault> {
        if address & 0x03 != 0 {
            return Err(BusFault::Unaligned { address });
        }

        if let Some(value) = self.read_mmio32(address) {
            return Ok(value);
        }

        self.read32(address)
    }

    fn write32(&mut self, address: u32, value: u32) -> Result<(), BusFault> {
        if address & 0x03 != 0 {
            return Err(BusFault::Unaligned { address });
        }

        if self.write_mmio32(address, value) {
            return Ok(());
        }

        for (offset, byte) in value.to_le_bytes().iter().copied().enumerate() {
            self.write_memory(address.wrapping_add(offset as u32), byte)?;
        }

        Ok(())
    }
}

fn apply_bshr(out: &mut u32, value: u32) {
    let set = value & 0xffff;
    let reset = (value >> 16) & 0xffff;
    *out |= set;
    *out &= !reset;
}

fn read_gpio(port: &GpioPort, base: u32, address: u32) -> Option<u32> {
    let offset = address.checked_sub(base)?;
    match offset {
        0x00 => Some(port.cfglr),
        0x08 | GPIO_OUT_OFFSET => Some(port.out),
        GPIO_BSHR_OFFSET | GPIO_BCR_OFFSET => Some(port.out),
        _ if offset < 0x18 => Some(0),
        _ => None,
    }
}

fn write_gpio(port: &mut GpioPort, base: u32, address: u32, value: u32) -> bool {
    let Some(offset) = address.checked_sub(base) else {
        return false;
    };

    match offset {
        0x00 => port.cfglr = value,
        GPIO_OUT_OFFSET => port.out = value,
        GPIO_BSHR_OFFSET => {
            port.bshr_writes = port.bshr_writes.wrapping_add(1);
            apply_bshr(&mut port.out, value);
        }
        GPIO_BCR_OFFSET => {
            port.bshr_writes = port.bshr_writes.wrapping_add(1);
            port.out &= !(value & 0xffff);
        }
        _ if offset < 0x18 => {}
        _ => return false,
    }

    true
}

fn push_byte(buffer: &mut [u8; SERIAL_BUFFER_BYTES], len: &mut usize, value: u8) {
    if *len < buffer.len() {
        buffer[*len] = value;
        *len += 1;
    }
}

fn transfer_width(bits: u32) -> u32 {
    match bits {
        1 => 2,
        2 => 4,
        _ => 1,
    }
}

fn is_dma_channel_control(address: u32) -> bool {
    if !(DMA1_BASE + 0x08..DMA1_BASE + 0x78).contains(&address) {
        return false;
    }

    (address - (DMA1_BASE + 0x08)) % 0x14 == 0
}

fn is_peripheral_mmio(address: u32) -> bool {
    matches!(
        address,
        TIM2_BASE..=0x4000_03ff
            | I2C1_BASE..=0x4000_57ff
            | AFIO_BASE..=0x4001_03ff
            | EXTI_BASE..=0x4001_07ff
            | GPIOA_BASE..=0x4001_0bff
            | GPIOC_BASE..=0x4001_13ff
            | GPIOD_BASE..=0x4001_17ff
            | ADC1_BASE..=0x4001_27ff
            | TIM1_BASE..=0x4001_2fff
            | SPI1_BASE..=0x4001_33ff
            | USART1_BASE..=0x4001_3bff
            | DMA1_BASE..=0x4002_03ff
            | 0x4002_1000..=0x4002_13ff
            | 0x4002_2000..=0x4002_23ff
            | 0xe000_e000..=0xe000_ffff
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_gpio_output() {
        let mut bus = RamBus::<4>::new();
        bus.write32(GPIO_OUT, 1).unwrap();
        assert_eq!(bus.gpio_out(), 1);
        assert_eq!(bus.read32(GPIO_OUT).unwrap(), 1);
    }

    #[test]
    fn rejects_unaligned_reads() {
        let bus = RamBus::<4>::new();
        assert_eq!(bus.read32(2), Err(BusFault::Unaligned { address: 2 }));
    }

    #[test]
    fn board_bus_tracks_all_one_dollar_board_gpio_ports() {
        let mut bus = BoardBus::<64, 64>::new();

        bus.write32(GPIOA_BASE + GPIO_BSHR_OFFSET, 1 << 1).unwrap();
        bus.write32(GPIOC_BASE + GPIO_BSHR_OFFSET, 1 << 7).unwrap();
        bus.write32(
            GPIOD_BASE + GPIO_BSHR_OFFSET,
            (1 << 0) | (1 << 1) | (1 << 2) | (1 << 6) | (1 << 7),
        )
        .unwrap();
        bus.write32(GPIOC_BASE + GPIO_BCR_OFFSET, 1 << 7).unwrap();

        assert_eq!(bus.gpio_a().out, 1 << 1);
        assert_eq!(bus.gpio_c().out, 0);
        assert_eq!(
            bus.gpio_d().out,
            (1 << 0) | (1 << 1) | (1 << 2) | (1 << 6) | (1 << 7)
        );
    }

    #[test]
    fn board_bus_captures_serial_peripheral_output() {
        let mut bus = BoardBus::<64, 64>::new();

        bus.write32(USART1_BASE + 0x04, b'A' as u32).unwrap();
        bus.write32(SPI1_BASE + 0x0c, 0x5a).unwrap();
        bus.write32(I2C1_BASE + 0x10, 0xc3).unwrap();

        assert_eq!(bus.usart1_tx(), b"A");
        assert_eq!(bus.spi1_tx(), &[0x5a]);
        assert_eq!(bus.i2c1_tx(), &[0xc3]);
    }

    #[test]
    fn board_bus_performs_basic_dma_mem2mem_copy() {
        let mut bus = BoardBus::<64, 64>::new();
        let src = BoardBus::<64, 64>::RAM_BASE;
        let dst = BoardBus::<64, 64>::RAM_BASE + 16;

        for (offset, value) in [9, 8, 7, 6].iter().copied().enumerate() {
            bus.write8(src + offset as u32, value).unwrap();
        }

        bus.write32(DMA1_BASE + 0x0c, 4).unwrap();
        bus.write32(DMA1_BASE + 0x10, src).unwrap();
        bus.write32(DMA1_BASE + 0x14, dst).unwrap();
        bus.write32(DMA1_BASE + 0x08, (1 << 14) | (1 << 7) | 1)
            .unwrap();

        assert_eq!(bus.dma_transfer_count(), 1);
        assert_eq!(bus.read8(dst).unwrap(), 9);
        assert_eq!(bus.read8(dst + 1).unwrap(), 8);
        assert_eq!(bus.read8(dst + 2).unwrap(), 7);
        assert_eq!(bus.read8(dst + 3).unwrap(), 6);
    }
}
