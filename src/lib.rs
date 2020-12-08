#![feature(const_generics)]

pub mod interpreter;
pub mod error;
pub mod types;
pub mod mem;
pub mod reg;
pub mod flags;

#[cfg(test)]
mod tests {
    #[test]
    fn cast() {
        let i = -1i32;
        let j = i as u32;

        println!("{}", j);
    }
}