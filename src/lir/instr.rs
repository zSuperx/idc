use crate::{ast::*, tir::Id};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegId(pub usize);

impl std::fmt::Display for RegId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("r{}", self.0))
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct Lbl(pub &'static str, pub usize);
impl std::fmt::Display for Lbl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}{}", self.0, self.1))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Instr {
    Function(&'static str),
    Alloc(Id, &'static str),
    Label(Lbl),
    Const {
        dst: RegId,
        imm: i128,
    },
    Copy {
        dst: RegId,
        rs1: RegId,
    },
    Bin {
        dst: RegId,
        op: BinOp,
        rs1: RegId,
        rs2: RegId,
    },
    Un {
        dst: RegId,
        op: UnOp,
        rs1: RegId,
    },
    AddrOf {
        dst: RegId,
        src: &'static str,
    },
    Read {
        dst: RegId,
        loc: &'static str,
    },
    Write {
        loc: &'static str,
        rs1: RegId,
    },
    Br {
        rs1: RegId,
        lbl1: Lbl,
        lbl2: Lbl,
    },
    Jmp {
        lbl: Lbl,
    },
    Arg {
        num: usize,
        rs1: RegId,
        ret: bool,
    },
    Call {
        loc: &'static str,
        dst: Option<RegId>,
    },
    Ret {
        rs1: Option<RegId>,
    },
}

impl std::fmt::Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[rustfmt::skip]
        let s = match self {
            Instr::Alloc(ty, name)              => format!("\talloc {ty} `{name}`"),
            Instr::Const { dst, imm }           => format!("\t{dst} = {imm}"),
            Instr::Copy { dst, rs1 }            => format!("\t{dst} = {rs1}"),
            Instr::Bin { dst, op, rs1, rs2 }    => format!("\t{dst} = {rs1} {op} {rs2}"),
            Instr::Un { dst, op, rs1 }          => format!("\t{dst} = {op} {rs1}"),
            Instr::AddrOf { dst, src }          => format!("\t{dst} = &{src}"),
            Instr::Read { dst, loc }            => format!("\t{dst} = load `{loc}`"),
            Instr::Write { loc, rs1 }           => format!("\tstore {rs1}, `{loc}`"),
            Instr::Label(lbl)                   => format!("{lbl}:"),
            Instr::Br { rs1, lbl1, lbl2  }      => format!("\tbr {rs1}, {lbl1}, {lbl2}"),
            Instr::Jmp { lbl }                  => format!("\tjmp {lbl}"),
            Instr::Function(name)               => format!(".F{name}:"),
            Instr::Arg { num, rs1, ret: true }  => format!("\targ#{num} = {rs1}"),
            Instr::Arg { num, rs1, ret: false } => format!("\tret#{num} = {rs1}"),
            Instr::Call { loc, dst: Some(dst) } => format!("\t{dst} = call {loc}"),
            Instr::Call { loc, dst: None }      => format!("\tcall {loc}"),
            Instr::Ret { rs1: Some(val) }       => format!("\tret {val}"),
            Instr::Ret { rs1: None }            => format!("\tret"),
        };
        f.write_str(&s)
    }
}
