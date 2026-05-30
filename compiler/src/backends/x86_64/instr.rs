use std::fmt::Display;

use iformatter::Iformat;

use crate::lir::{BB, VReg};

#[derive(Clone, Copy)]
pub enum Val {
    Imm(i128),
    Reg(Reg),
    Offset(Reg, i128),
}

impl Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Imm(imm) => f.write_fmt(format_args!("{imm}")),
            Val::Reg(reg) => f.write_fmt(format_args!("{reg}")),
            Val::Offset(reg, imm) => match imm {
                ..0 => f.write_fmt(format_args!("[{reg} - {}]", imm.abs())),
                 0  => f.write_fmt(format_args!("[{reg}]")),
                0.. => f.write_fmt(format_args!("[{reg} + {}]", imm.abs())),
            },
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg {
    Virt(VReg),
    Rdi, // arg 1
    Rsi, // arg 2
    Rdx, // arg 3
    Rcx, // arg 4
    R8,  // arg 5
    R9,  // arg 6

    // Temporary registers that functions may change
    Rax, // Return value
    R10,
    R11, // (i use this for temp shit like cmovcc)

    // Callee-saved registers that will stay unchanged
    Rsp, // Stack pointer
    Rbp, // Frame pointer
    Rbx, // Base pointer (i dont treat it like one)
    R12, // (i use this to store the heap pointer)
    R13, // (i use this to store the end of the heap)
    R14,
    R15,
}

impl std::fmt::Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reg::Virt(reg_id) => f.write_fmt(format_args!("{reg_id}")),
            _ => f.write_str(&format!("{self:?}").to_ascii_lowercase()),
        }
    }
}

#[derive(Iformat)]
pub enum Instr {
    /// %1
    Raw(String),
    /// ; %1
    Comment(String),

    // Moves
    Lea(Val, Val),
    Mov(Val, Val),
    Push(Val),
    Pop(Val),

    // Arithmetic
    IMul(Val, Val),
    Mul(Val, Val),
    Sub(Val, Val),
    Add(Val, Val),

    // Function calls
    Call(String),
    /// call %1
    ICall(Val),
    Ret,

    // Bitwise operations
    And(Val, Val),
    Or(Val, Val),
    Test(Val, Val),
    Xor(Val, Val),
    Sar(Val, Val),
    Sal(Val, Val),

    // Conditional moves
    Cmove(Val, Val),
    Cmovne(Val, Val),
    Cmovz(Val, Val),
    Cmovnz(Val, Val),
    Cmovg(Val, Val),
    Cmovge(Val, Val),
    Cmovl(Val, Val),
    Cmovle(Val, Val),
    Cmp(Val, Val),

    // Jump/Branch instructions
    /// %1:
    Label(BB),
    Je(BB),
    Jne(BB),
    Jz(BB),
    Jnz(BB),
    Jg(BB),
    Jge(BB),
    Jl(BB),
    Jle(BB),
    Jo(BB),
    Jc(BB),
    Js(BB),
    Jb(BB),
    Jns(BB),
    Jmp(BB),
}
