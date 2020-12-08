use crate::reg::{Register as R, Register};
use crate::types::Immediate as I;
use crate::types::Function as F;
use crate::types::Location as D;
use crate::mem::{Pointer as M, Pointer};
use crate::interpreter::{Executable, Environment, Interpreter, alias::*};
use crate::error::{Result, EOF_ERR, Error, ErrorKind, CODE_ERR};

use std::convert::{TryFrom, TryInto};

#[derive(Debug, Copy, Clone)]
pub enum Code {
    Halt,                   // 00
    Nop,                    // 10
    CondMove(F, R, R),      // condition move, only register-register, 2x, moving immediately if F equals 0
    IRMove(I, R),           // move immediate to register, 30
    RMMove(R, M),           // move register to memory, 40
    MRMove(M, R),           // move memory to register, 50
    OP(F, R, R),            // execute some operation (add, sub...), only between register and register, 6x
    Jump(F, D),             // jump with some condition (equal, less...), 7x
    Call(D),                // Call a function, 80
    Return,                 // Return, 90
    Push(R),                // Push a register to the stack, A0
    Pop(R)                  // Pop the stack top to a register, B0
}

impl Executable for Code {
    fn execute(&self, itpt: &mut Interpreter) -> Result<()> {
        let mut env = &mut itpt.env;

        match self {
            Code::Halt => {
                return Err(Error { kind: ErrorKind::Halt })
            }
            Code::Nop => (),         // do nothing
            Code::CondMove(f, r0, r1) => {
                let r0 = *r0;
                let r1 = *r1;
                match *f {
                    0 => env.registers[r1] = env.registers[r0],
                    _ => unimplemented!()
                }
            }
            Code::IRMove(i, r) => {
                env.registers[*r] = *i;
            }
            Code::RMMove(r, m) => {
                let bytes = env.registers[*r];
                m.write(bytes, env)?;       // use '?'!
            }
            Code::MRMove(m, r) => {
                let bytes = m.read::<u64>(env)?;
                env.registers[*r] = bytes;
            }
            Code::OP(f, r_a, r_b) => {
                let v_a = env.registers[*r_a];
                let v_b = env.registers[*r_b];

                let result = match *f {
                    0 => v_b.wrapping_add(v_a),
                    1 => v_b.wrapping_sub(v_a),
                    2 => v_b & v_a,
                    3 => v_b ^ v_a,

                    _ => unreachable!()
                };

                env.registers[*r_b] = result;
            }

            Code::Jump(f, addr) => {
                match *f {
                    0 => {
                        itpt.pc = *addr;
                    },
                    _ => unimplemented!()
                }
            }
            Code::Call(_) => {}
            Code::Return => {}
            Code::Push(_) => {}
            Code::Pop(_) => {}
        }

        Ok(())
    }
}

impl Code {
    pub fn parse(itpt: &mut Interpreter) -> Result<Code> {
        fn register(itpt: &mut Interpreter) -> Result<(Register, Register)> {
            let regs = *itpt.read(1)?.first().unwrap();
            let r_a = (regs & 0xF0) >> 4;
            let r_b = regs & 0x0F;

            Ok((Register::try_from(r_a)?, Register::try_from(r_b)?))
        }

        fn immediate(itpt: &mut Interpreter) -> Result<I> {
            let bytes: [u8; 8] = itpt.read(8)?.as_slice().try_into().ok().unwrap();

            Ok(u64::from_le_bytes(bytes))
        }

        let code = *itpt.read(1)?.first().unwrap();

        match code {
            HALT => Ok(Code::Halt),
            NOP  => Ok(Code::Nop),
            RRMOV | 0x21 /* 0x22 ... */ => {
                let (r_a, r_b) = register(itpt)?;
                Ok(Code::CondMove(0x0, r_a, r_b))
            },

            IRMOV => {
                let (_, r_b) = register(itpt)?;
                let imm = immediate(itpt)?;
                Ok(Code::IRMove(imm, r_b))
            }

            RMMOV => {
                let (r_a, r_b) = register(itpt)?;
                let dest = immediate(itpt)?;
                Ok(Code::RMMove(r_a, Pointer::new(r_b, dest as i64)))
            }

            MRMOV => {
                let (r_a, r_b) = register(itpt)?;
                let dest = immediate(itpt)?;

                Ok(Code::MRMove(Pointer::new(r_b, dest as i64), r_a))
            }

            ADD | 0x61 | 0x62 | 0x63 => {
                let f = code & 0xF;
                let (r_a, r_b) = register(itpt)?;

                Ok(Code::OP(f, r_a, r_b))
            }

            _ => Err(CODE_ERR)
        }
    }
}