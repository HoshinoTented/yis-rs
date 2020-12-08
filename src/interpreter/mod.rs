pub mod code;
pub mod interpreter;
pub mod alias;

use crate::reg::{Registers, Register};
use crate::mem::{Stack, Pointer};
use crate::error::{Result, REG_ERR};
use crate::interpreter::interpreter::Interpreter;

#[derive(Debug)]
pub struct Environment {
    pub registers: Registers,
    pub memory: Stack
}

impl Environment {
    pub fn get_mem(&self, addr: Pointer) -> Result<u8> {
        addr.read(self)
    }

    pub fn get_reg(&self, reg: Register) -> Result<u64> {
        if reg == Register::None {
            Err(REG_ERR)
        } else {
            Ok(self.registers[reg])
        }
    }
}

pub trait Executable {
    fn execute(&self, env: &mut Interpreter) -> Result<()>;
}