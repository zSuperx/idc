#![allow(unused)]

use iformatter::Iformat;

#[derive(Iformat)]
#[valueType(i32)]
enum Instr {
    /// dst: %1
    /// src: %2
    /// fmt: r%1, r%2
    Add(i32, i32),

    Jmp(&'static str),

    /// dst: %1
    /// src: %2, %3
    Addi(i32, i32, i32),
    /// fmt: r%1 ? %2 : %3
    Br(i32, &'static str, &'static str),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let i = Instr::Add(12, 69);
        println!("{i}");
        println!("{:?}, {:?}", i.dsts(), i.srcs());

        let i = Instr::Br(69, "then", "else");
        println!("{i}");
        println!("{:?}, {:?}", i.dsts(), i.srcs());

        let i = Instr::Addi(1, 2, 3);
        println!("{i}");
        println!("{:?}, {:?}", i.dsts(), i.srcs());
    }
}
