pub const GPIO_BASE: u32 = 0x4001_1000;
pub const GPIO_OUT: u32 = GPIO_BASE + 0x0c;
pub const GPIOC_BASE: u32 = 0x4001_1000;
pub const GPIOD_BASE: u32 = 0x4001_1400;
pub const GPIO_BSHR_OFFSET: u32 = 0x10;

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
    gpio_c: GpioPort,
    gpio_d: GpioPort,
    rcc_ctlr: u32,
    rcc_cfgr0: u32,
    rcc_apb2pcenr: u32,
    flash_actlr: u32,
    systick_ctlr: u32,
    systick_count: u32,
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
            rcc_apb2pcenr: 0,
            flash_actlr: 0,
            systick_ctlr: 0,
            systick_count: 0,
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

    pub const fn gpio_c(&self) -> GpioPort {
        self.gpio_c
    }

    pub const fn gpio_d(&self) -> GpioPort {
        self.gpio_d
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
        match address {
            0xe000_f000 => Some(self.systick_ctlr),
            0xe000_f008 => {
                self.systick_count = self.systick_count.wrapping_add(50_000);
                Some(self.systick_count)
            }
            0x4002_1000 => Some(self.rcc_ctlr | (1 << 25)),
            0x4002_1004 => Some((self.rcc_cfgr0 & !0x0c) | 0x08),
            0x4002_1008 => Some(0),
            0x4002_1018 => Some(self.rcc_apb2pcenr),
            0x4002_2000 => Some(self.flash_actlr),
            GPIOC_BASE => Some(self.gpio_c.cfglr),
            GPIOD_BASE => Some(self.gpio_d.cfglr),
            address if address == GPIOC_BASE + GPIO_BSHR_OFFSET => Some(self.gpio_c.out),
            address if address == GPIOD_BASE + GPIO_BSHR_OFFSET => Some(self.gpio_d.out),
            _ => None,
        }
    }

    fn write_mmio32(&mut self, address: u32, value: u32) -> bool {
        match address {
            0xe000_f000 => self.systick_ctlr = value,
            0x4002_1000 => self.rcc_ctlr = value | (1 << 25),
            0x4002_1004 => self.rcc_cfgr0 = (value & !0x0c) | 0x08,
            0x4002_1008 => {}
            0x4002_1018 => self.rcc_apb2pcenr = value,
            0x4002_2000 => self.flash_actlr = value,
            GPIOC_BASE => self.gpio_c.cfglr = value,
            GPIOD_BASE => self.gpio_d.cfglr = value,
            address if address == GPIOC_BASE + GPIO_BSHR_OFFSET => {
                self.gpio_c.bshr_writes = self.gpio_c.bshr_writes.wrapping_add(1);
                apply_bshr(&mut self.gpio_c.out, value);
            }
            address if address == GPIOD_BASE + GPIO_BSHR_OFFSET => {
                self.gpio_d.bshr_writes = self.gpio_d.bshr_writes.wrapping_add(1);
                apply_bshr(&mut self.gpio_d.out, value);
            }
            _ => return false,
        }

        true
    }
}

impl<const FLASH: usize, const RAM: usize> Default for BoardBus<FLASH, RAM> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const FLASH: usize, const RAM: usize> Bus for BoardBus<FLASH, RAM> {
    fn read8(&mut self, address: u32) -> Result<u8, BusFault> {
        self.read_memory(address)
    }

    fn write8(&mut self, address: u32, value: u8) -> Result<(), BusFault> {
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
}
