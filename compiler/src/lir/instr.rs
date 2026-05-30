use iformatter::Iformat;

use crate::{
    ast::*,
    lir::{BB, LirType},
    tir::{TypeId, VarId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VReg(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VVal {
    Ptr(VReg),
    Reg(VReg),
}

impl std::fmt::Display for VReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("%{}", self.0))
    }
}

#[derive(Debug, Clone, Copy, Iformat)]
pub enum Instr {
    Param(LirType, VReg, usize, VarId),
    Alloc(LirType, VReg, VarId),

    Const(LirType, VReg, i128),
    Copy(LirType, VReg, VReg),

    Add(LirType, VReg, VReg, VReg),
    Sub(LirType, VReg, VReg, VReg),
    Muls(LirType, VReg, VReg, VReg),
    Mulu(LirType, VReg, VReg, VReg),

    Eq(LirType, VReg, VReg, VReg),

    Sgt(LirType, VReg, VReg, VReg),
    Sge(LirType, VReg, VReg, VReg),
    Slt(LirType, VReg, VReg, VReg),
    Sle(LirType, VReg, VReg, VReg),

    Ugt(LirType, VReg, VReg, VReg),
    Uge(LirType, VReg, VReg, VReg),
    Ult(LirType, VReg, VReg, VReg),
    Ule(LirType, VReg, VReg, VReg),

    Load(LirType, VReg, VReg),
    Store(LirType, VReg, VReg),

    Br(VReg, BB, BB),
    Jmp(BB),
    Ret(LirType, VReg),
    /// ret
    RetVoid,
}

