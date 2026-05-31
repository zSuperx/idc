use std::fmt::Display;

use iformatter::Iformat;

use crate::lir::{BB, Instr, LirType, LirVal};

#[derive(Clone, Copy)]
pub enum Val {
    Imm(i128),
    Reg(Reg),
    Offset(LirType, Reg, i128),
}

impl Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Imm(imm) => f.write_fmt(format_args!("{imm}")),
            Val::Reg(reg) => f.write_fmt(format_args!("{reg}")),
            Val::Offset(ty, reg, imm) => {
                let width_spec = match ty.size() {
                    1 => "byte",
                    2 => "word",
                    4 => "dword",
                    8 => "qword",
                    _ => unreachable!(),
                };
                match imm {
                    ..0 => f.write_fmt(format_args!("{width_spec} [{reg} - {}]", imm.abs())),
                    0 => f.write_fmt(format_args!("{width_spec} [{reg}]")),
                    0.. => f.write_fmt(format_args!("{width_spec} [{reg} + {}]", imm.abs())),
                }
            }
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg {
    Virt(usize),
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
            Reg::Virt(reg_id) => f.write_fmt(format_args!("%{reg_id}")),
            _ => f.write_str(&format!("{self:?}").to_ascii_lowercase()),
        }
    }
}

#[derive(Iformat, Clone)]
pub enum x86Instr {
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

impl Instr for x86Instr {
    fn is_terminator(&self) -> bool {
        match self {
            x86Instr::Ret
            | x86Instr::Je(..)
            | x86Instr::Jne(..)
            | x86Instr::Jz(..)
            | x86Instr::Jnz(..)
            | x86Instr::Jg(..)
            | x86Instr::Jge(..)
            | x86Instr::Jl(..)
            | x86Instr::Jle(..)
            | x86Instr::Jo(..)
            | x86Instr::Jc(..)
            | x86Instr::Js(..)
            | x86Instr::Jb(..)
            | x86Instr::Jns(..)
            | x86Instr::Jmp(..) => true,
            _ => false,
        }
    }

    fn uncond_jump(target: BB) -> Self {
        Self::Jmp(target)
    }
}
