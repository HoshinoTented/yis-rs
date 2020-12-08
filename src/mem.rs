use super::types::Offset;
use crate::reg::{Registers, Register};
use crate::error::{Result, ErrorKind, Error, MEMORY_ERR};
use crate::interpreter::Environment;

use std::ops::{Add, Shl, Sub, BitOr, Index, IndexMut};
use std::mem::size_of;
use std::array::TryFromSliceError;
use std::convert::TryInto;
use std::slice::SliceIndex;

#[derive(Debug, Copy, Clone)]
pub struct Pointer {
    base: Register,
    // always rB
    offset: Offset,
}

impl Pointer {
    pub fn new(base: Register, offset: Offset) -> Self {
        Pointer {
            base,
            offset,
        }
    }

    pub fn base(&self) -> Register {
        self.base
    }

    pub fn offset(&self) -> Offset {
        self.offset
    }

    pub fn read<T>(&self, env: &Environment) -> Result<T>
        where T: TryFromBytes {
        let base = self.abs(&env.registers) as usize;
        let len: usize = size_of::<T>();

        if env.memory.len() >= base + len {     // ensure that memory has enough bytes
            let mut bytes = vec![0u8; len];

            for i in 0..len {
                bytes[i] = env.memory[base + i];
            }

            Ok(T::try_from_bytes(bytes.as_slice()).ok_or(MEMORY_ERR)?)
        } else {
            Err(MEMORY_ERR)
        }
    }

    pub fn write<T>(&self, value: T, env: &mut Environment) -> Result<()>
        where T: IntoBytes {
        let base = self.abs(&env.registers) as usize;
        let bytes = value.into_le_bytes();
        let len = bytes.len();

        if env.memory.len() >= base + len {
            for i in 0..len {
                env.memory[base + i] = bytes[i];
            }

            Ok(())
        } else {
            Err(MEMORY_ERR)
        }
    }

    pub fn abs(&self, env: &Registers) -> u64 {
        let Self {
            base, offset
        } = *self;

        let offset = offset as u64;
        let base = if base == Register::None {
            0
        } else {
            env[base]
        };

        base.wrapping_add(offset)
    }
}

impl Add<i64> for Pointer {
    type Output = Pointer;

    fn add(self, rhs: i64) -> Self::Output {
        Pointer {
            offset: self.offset + rhs,
            ..self
        }
    }
}

impl Sub<i64> for Pointer {
    type Output = Pointer;

    fn sub(self, rhs: i64) -> Self::Output {
        Pointer {
            offset: self.offset - rhs,
            ..self
        }
    }
}

#[derive(Debug)]
pub struct Stack {
    top: usize,
    stack: Box<[u8]>,
}

impl Stack {
    pub fn new(size: usize) -> Self {
        Stack {
            top: size,
            stack: Box::from(vec![0u8; size]),
        }
    }

    pub fn push(&mut self, value: u8) -> Result<()> {
        if self.top >= 0 {
            self.top -= 1;
            self.stack[self.top] = value;

            Ok(())
        } else {
            Err(MEMORY_ERR)
        }
    }

    pub fn push_n<const N: usize>(&mut self, value: [u8; N]) -> Result<()> {
        if self.unused() >= N {
            self.top -= N;

            for i in 0..N {
                let top = self.top;
                self[top + i] = value[i];
            }

            Ok(())
        } else {
            Err(MEMORY_ERR)
        }
    }

    pub fn pop(&mut self) -> Result<u8> {
        if self.top < self.stack.len() {
            let value = self.stack[self.top];
            self.top += 1;

            Ok(value)
        } else {
            Err(MEMORY_ERR)
        }
    }

    pub fn get(&self, index: usize) -> Option<&u8> {
        self.stack.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
        self.stack.get_mut(index)
    }

    pub fn unused(&self) -> usize {
        self.top
    }

    pub fn used(&self) -> usize {
        self.stack.len() - self.top
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.stack
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.stack
    }
}

impl<I> Index<I> for Stack
    where I: SliceIndex<[u8]> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(self.as_slice(), index)
    }
}

impl<I> IndexMut<I> for Stack
    where I: SliceIndex<[u8]> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(self.as_mut_slice(), index)
    }
}

#[cfg(test)]
mod tests {
    use crate::mem::{Pointer, Stack};
    use crate::reg::{Registers, Register};
    use crate::interpreter::Environment;
    use crate::reg::Register::RAX;

    fn env() -> Registers {
        let mut regs = Registers::default();
        regs[RAX] = 0xF;

        regs
    }

    #[test]
    fn memory() {
        let env = env();
        let mem = Pointer::new(Register::RAX, 8);
        assert_eq!(0x17, mem.abs(&env));
    }

    #[test]
    fn stack() {
        let mut stack = Stack::new(4);

        assert!(stack.pop().is_err());

        stack.push(0x1);
        assert_eq!(Some(0x1), stack.pop().ok());

        for _ in 0..4 {
            stack.push(0x1);
        }

        assert!(stack.push(0x0).is_err());
    }

    #[test]
    fn test_rw() {
        let mut stack = Stack::new(4);
        let regs = Registers::default();
        let mut env = Environment {
            registers: regs,
            memory: stack,
        };

        let ptr = Pointer::new(Register::RAX, 0);

        assert_eq!(Some(()), ptr.write(0x12345678u32, &mut env).ok());
        assert_eq!(Some(0x12345678u32), ptr.read(&env).ok());
    }

    #[test]
    fn push_c() {
        let mut stack = Stack::new(4);
        let mut regs = Registers::default();
        let mut env = Environment {
            registers: regs,
            memory: stack,
        };

        assert_eq!(Some(()), env.memory.push_n([0x78, 0x56, 0x34, 0x12]).ok());

        let ptr = Pointer::new(Register::RAX, 0);

        assert_eq!(Some(0x12345678u32), ptr.read(&env).ok());
    }
}

pub trait TryFromBytes where Self: Sized {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self>;
}

pub trait IntoBytes where Self: Sized {
    fn into_le_bytes(self) -> Box<[u8]>;
}

macro_rules! from_bytes_impl {
    ($type:ty; $len:literal) => {
        impl TryFromBytes for $type {
            fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
                Some(<$type>::from_le_bytes(bytes.try_into().ok()?))
            }
        }
    };
}

macro_rules! into_bytes_impl {
    ($type:ty) => {
        impl IntoBytes for $type {
            fn into_le_bytes(self) -> Box<[u8]> {
                let bytes = self.to_le_bytes();
                Box::new(bytes)
            }
        }
    };
}

from_bytes_impl!(u8; 1);
from_bytes_impl!(u32; 4);
from_bytes_impl!(u64; 8);
into_bytes_impl!(u32);
into_bytes_impl!(u64);