pub const REGISTER_COUNT: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegisterError {
    NotRv32e(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Register(u8);

impl Register {
    pub const ZERO: Self = Self(0);
    pub const RETURN_ADDRESS: Self = Self(1);
    pub const STACK_POINTER: Self = Self(2);

    pub const fn new(index: u8) -> Result<Self, RegisterError> {
        if index < REGISTER_COUNT as u8 {
            Ok(Self(index))
        } else {
            Err(RegisterError::NotRv32e(index))
        }
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }

    pub const fn raw(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_rv32e_registers() {
        assert_eq!(Register::new(15).unwrap().index(), 15);
        assert_eq!(Register::new(16), Err(RegisterError::NotRv32e(16)));
        assert_eq!(Register::new(31), Err(RegisterError::NotRv32e(31)));
    }
}
