#![allow(unused)]

use iformatter::Iformat;

#[derive(Iformat)]
enum Instr {
    /// add %1, %2
    Add(i32, i32),
    /// ret
    Ret,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let i = Instr::Ret;
        println!("{i}");
        let i = Instr::Add(12, 69);
        println!("{i}");
    }
}
