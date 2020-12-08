use crate::interpreter::{Environment, Executable};
use crate::interpreter::code::Code;
use crate::error::{Result, Error, ErrorKind, EOF_ERR};

#[derive(Debug)]
pub struct Interpreter {
    pub pc: u64,
    pub env: Environment,
    pub source: Vec<u8>
}

impl Interpreter {
    pub fn execute(&mut self) -> Result<()> {
        while self.pc < self.source.len() as u64 {
            match Code::parse(self) {
                Ok(ins) => {
                    ins.execute(self)?;
                },

                Err(Error { kind: ErrorKind::EOF }) => {
                    return Ok(());
                },

                otherwise => {
                    otherwise?;
                }
            }
        }

        Ok(())
    }

    pub fn read(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(8);
        let end = self.pc + (size as u64);

        for pc in self.pc..end {
            let byte = *self.source.get(pc as usize).ok_or(EOF_ERR)?;
            bytes.push(byte);
        }

        self.pc = end;

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::interpreter::Interpreter;
    use crate::interpreter::Environment;
    use crate::mem::Stack;
    use crate::reg::Registers;

    #[test]
    fn test0() {
        let code = [
            0x30u8, 0xF0, 0x78, 0x56, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, // irmov $12345678, %rax
            0x40u8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // rmmov %rax, 0(%rcx)
            0x50u8, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mrmov 0(%rcx), %rcx
            0x30u8, 0xF0, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // irmov $1, %rax
            0x30u8, 0xF2, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // irmov $2, %rdx
            0x60u8, 0x02,                                                 // addq %rax, %rdx
        ];

        let mut itpt = Interpreter {
            pc: 0,
            env: Environment {
                registers: Registers::default(),
                memory: Stack::new(16)
            },
            source: code.to_vec()
        };

        itpt.execute().unwrap();

        println!("{:?}", itpt);
    }
}