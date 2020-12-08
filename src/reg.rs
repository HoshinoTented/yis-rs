use std::convert::TryFrom;
use crate::error::{Error, ErrorKind};
use std::ops::{Index, IndexMut};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Register {
    RAX = 0,
    RCX,
    RDX,
    RBX,
    RSP,
    RBQ,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    None,
}

impl TryFrom<u8> for Register {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Register::*;

        const REGS: [Register; 16] = [RAX, RCX, RDX, RBX, RSP, RBQ, RSI, RDI, R8, R9, R10, R11, R12, R13, R14, None];
        REGS.get(value as usize)
            .map(Register::clone)
            .ok_or(Error { kind: ErrorKind::Register })
    }
}

#[derive(Debug)]
pub struct Registers(pub [u64; 16]);

impl Index<Register> for Registers {
    type Output = u64;

    fn index(&self, index: Register) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl Default for Registers {
    fn default() -> Self {
        Registers([0u64; 16])
    }
}

// pub type Registers = [u64; 16];