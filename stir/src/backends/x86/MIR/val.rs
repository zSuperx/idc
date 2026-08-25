use std::fmt::Display;

use super::reg::*;

#[derive(Clone, Debug, Copy)]
pub enum RFLAG {
    LT,
    LE,
    GT,
    GE,
    EQ,
    NE,
    ZF,
    NZ,
    OF,
    SF,
}

impl Display for RFLAG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{self:?}").to_lowercase();
        f.write_str(s.as_str())
    }
}

#[derive(Clone, Debug, Copy)]
pub enum x86Val {
    Imm(i128),
    Reg {
        reg: Reg,
        size: usize,
    },
    Address {
        base: Reg,
        offset: i128,
        size: usize,
    },
    CC(RFLAG),
}

impl x86Val {
    pub const fn reg(reg: Reg, size: usize) -> x86Val {
        x86Val::Reg { reg, size }
    }

    pub const fn addr(reg: Reg, offset: i128, size: usize) -> x86Val {
        x86Val::Address {
            base: reg,
            offset,
            size,
        }
    }
}

impl Display for x86Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            x86Val::Imm(i) => f.write_fmt(format_args!("{i}")),
            x86Val::Reg { reg, size } => reg.sized_print(f, *size),
            x86Val::Address { base, offset, size } => {
                let width_spec = match size {
                    8 => "byte",
                    16 => "word",
                    32 => "dword",
                    64 => "qword",
                    s => panic!("Unsupported size: {s}"),
                };
                f.write_fmt(format_args!("{width_spec} ["))?;
                base.sized_print(f, 64)?;
                match offset {
                    ..0 => f.write_fmt(format_args!(" - {}", offset.abs()))?,
                    0 => {}
                    1.. => f.write_fmt(format_args!(" + {offset}"))?,
                }
                f.write_str("]")
            }
            x86Val::CC(flags) => flags.fmt(f),
        }
    }
}

pub const RBP: x86Val = x86Val::Reg {
    reg: Reg::BP,
    size: 64,
};

pub const RSP: x86Val = x86Val::Reg {
    reg: Reg::SP,
    size: 64,
};
