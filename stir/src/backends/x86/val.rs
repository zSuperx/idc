use std::fmt::Display;

use super::reg::*;
use super::types::*;

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum RFLAG {
    LT,
    LE,
    GT,
    GE,
    EQ,
    NE,

    /// Zero
    Z,
    /// Not zero
    NZ,

    /// Overflow
    O,
    /// No overflow
    NO,
}

impl Display for RFLAG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{self:?}").to_lowercase();
        f.write_str(s.as_str())
    }
}

#[derive(Clone, Debug, Copy)]
pub enum x86Value {
    Imm(i128),
    Reg {
        name: Reg,
        ty: LLType,
    },
    Mem {
        base: Reg,
        index: Option<Reg>,
        scale: usize,
        disp: i128,
        ty: LLType,
    },
    CC(RFLAG),
}

impl x86Value {
    pub const fn reg(name: Reg, ty: LLType) -> x86Value {
        x86Value::Reg { name, ty }
    }

    pub const fn mem(base: Reg, ty: LLType) -> x86Value {
        x86Value::Mem {
            base,
            index: None,
            scale: 1,
            disp: 0,
            ty,
        }
    }

    pub fn getReg(&self) -> Reg {
        let x86Value::Reg { name, .. } = self else {
            panic!("This method can only be called on registers");
        };
        *name
    }

    pub const fn memDisp(base: Reg, disp: i128, ty: LLType) -> x86Value {
        x86Value::Mem {
            base,
            index: None,
            scale: 1,
            disp,
            ty,
        }
    }

    pub const fn memFull(
        base: Reg,
        index: Option<Reg>,
        scale: usize,
        disp: i128,
        ty: LLType,
    ) -> x86Value {
        x86Value::Mem {
            base,
            index,
            scale,
            disp,
            ty,
        }
    }

    pub fn ty(&self) -> LLType {
        match self {
            x86Value::CC(_) | x86Value::Imm(_) => panic!("{self} value does not have a size"),
            x86Value::Reg { name, ty } => *ty,
            x86Value::Mem { ty, .. } => *ty,
        }
    }
}

impl Display for x86Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            x86Value::Imm(i) => i.fmt(f),
            x86Value::Reg { name, ty } => name.sized_print(f, ty.bits()),
            x86Value::Mem {
                base,
                ty,
                index,
                scale,
                disp,
            } => {
                f.write_str(ty.width_str())?;
                f.write_str("[")?;
                base.sized_print(f, 64);
                if let Some(i) = index {
                    assert_ne!(*i, Reg::SP);
                    f.write_str(" + ")?;
                    i.sized_print(f, 64)?;
                    if *scale > 1 {
                        f.write_fmt(format_args!("*{scale}"))?;
                    }
                }

                match disp {
                    ..0 => f.write_fmt(format_args!(" - {}", disp.abs()))?,
                    0 => {}
                    1.. => f.write_fmt(format_args!(" + {}", disp.abs()))?,
                }
                f.write_str("]")
            }
            x86Value::CC(flags) => flags.fmt(f),
        }
    }
}
