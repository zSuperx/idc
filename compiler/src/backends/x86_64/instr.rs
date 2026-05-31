use std::fmt::Display;

use iformatter::Iformat;

use crate::lir::{BB, Instr, LirType, LirVal};

#[derive(Clone, Copy)]
pub enum x86Val {
    Imm(i128),
    Reg(x86Reg),
    Offset(LirType, x86Reg, i128),
}

impl Display for x86Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            x86Val::Imm(imm) => f.write_fmt(format_args!("{imm}")),
            x86Val::Reg(reg) => f.write_fmt(format_args!("{reg}")),
            x86Val::Offset(ty, reg, imm) => {
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
pub enum x86Reg {
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
    R11,

    // Callee-saved registers that will stay unchanged
    Rsp, // Stack pointer
    Rbp, // Frame pointer
    Rbx, // Base pointer
    R12,
    R13,
    R14,
    R15,
}

impl std::fmt::Display for x86Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            x86Reg::Virt(reg_id) => f.write_fmt(format_args!("%{reg_id}")),
            _ => f.write_str(&format!("{self:?}").to_ascii_lowercase()),
        }
    }
}

#[derive(Iformat, Clone)]
#[valueType(x86Val)]
pub enum x86Instr {
    /// %1
    Raw(String),
    /// ; %1
    Comment(String),

    // Moves
    Lea(x86Val, x86Val),
    Mov(x86Val, x86Val),
    Push(x86Val),
    Pop(x86Val),

    // Arithmetic
    IMul(x86Val, x86Val),
    Mul(x86Val, x86Val),
    Sub(x86Val, x86Val),
    Add(x86Val, x86Val),

    // Function calls
    Call(String),
    /// call %1
    ICall(x86Val),
    Ret,

    // Bitwise operations
    And(x86Val, x86Val),
    Or(x86Val, x86Val),
    Test(x86Val, x86Val),
    Xor(x86Val, x86Val),
    Sar(x86Val, x86Val),
    Sal(x86Val, x86Val),

    // Conditional moves
    Cmove(x86Val, x86Val),
    Cmovne(x86Val, x86Val),
    Cmovz(x86Val, x86Val),
    Cmovnz(x86Val, x86Val),
    Cmovg(x86Val, x86Val),
    Cmovge(x86Val, x86Val),
    Cmovl(x86Val, x86Val),
    Cmovle(x86Val, x86Val),
    Cmp(x86Val, x86Val),

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
