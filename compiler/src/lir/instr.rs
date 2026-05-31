use iformatter::Iformat;

use crate::{
    ast::*,
    lir::{BB, LirType},
    tir::{TypeId, VarId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LirVal {
    Reg(usize),
    Imm(i128),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VVal {
    Ptr(LirVal),
    Reg(LirVal),
}

impl std::fmt::Display for LirVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LirVal::Reg(r) => f.write_fmt(format_args!("%{r}")),
            LirVal::Imm(i) => f.write_fmt(format_args!("{i}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Iformat)]
#[valueType(LirVal)]
pub enum LirInstr {
    Param(LirType, LirVal, usize, VarId),
    Alloc(LirType, LirVal, VarId),

    Copy(LirType, LirVal, LirVal),

    Add(LirType, LirVal, LirVal, LirVal),
    Sub(LirType, LirVal, LirVal, LirVal),
    Muls(LirType, LirVal, LirVal, LirVal),
    Mulu(LirType, LirVal, LirVal, LirVal),

    Eq(LirType, LirVal, LirVal, LirVal),

    Sgt(LirType, LirVal, LirVal, LirVal),
    Sge(LirType, LirVal, LirVal, LirVal),
    Slt(LirType, LirVal, LirVal, LirVal),
    Sle(LirType, LirVal, LirVal, LirVal),

    Ugt(LirType, LirVal, LirVal, LirVal),
    Uge(LirType, LirVal, LirVal, LirVal),
    Ult(LirType, LirVal, LirVal, LirVal),
    Ule(LirType, LirVal, LirVal, LirVal),

    Load(LirType, LirVal, LirVal),
    Store(LirType, LirVal, LirVal),

    Br(LirVal, BB, BB),
    Jmp(BB),
    Ret(LirType, LirVal),
    /// fmt: ret
    RetVoid,
}

impl LirInstr {
    pub fn is_ret(&self) -> bool {
        matches!(self, LirInstr::Ret(..) | LirInstr::RetVoid)
    }
}

impl Instr for LirInstr {
    fn is_terminator(&self) -> bool {
        match self {
            Self::Jmp(..) | Self::Br(..) | Self::RetVoid | Self::Ret(..) => true,
            _ => false,
        }
    }

    fn uncond_jump(target: BB) -> Self {
        Self::Jmp(target)
    }
}

pub trait Instr {
    fn is_terminator(&self) -> bool;

    fn uncond_jump(target: BB) -> Self;
}
