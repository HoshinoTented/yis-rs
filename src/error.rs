use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) const MEMORY_ERR: Error = Error { kind: ErrorKind::Memory };
pub(crate) const EOF_ERR: Error = Error { kind: ErrorKind::EOF };
pub(crate) const REG_ERR: Error = Error { kind: ErrorKind::Register };
pub(crate) const CODE_ERR: Error = Error { kind: ErrorKind::Code };

#[derive(Debug)]
pub enum ErrorKind {
    Register,
    Memory,
    Code,
    Halt,
    EOF
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}