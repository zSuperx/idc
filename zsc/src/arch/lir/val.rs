#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LirVal {
    pub kind: LirValKind,
    /// Size of value (in bytes).
    ///
    /// This means the size that the LirVal is pointing to. In the case of LirValKind::Reg, it's the
    /// size of the value the register holds (analogous to the sizes of x86 registers). If it's a
    /// LirValKind::Mem, the size refers to the size of pointed to object
    pub size: usize,
}

impl LirVal {
    pub fn imm(val: i128, size: usize) -> Self {
        LirVal {
            kind: LirValKind::Imm(val),
            size,
        }
    }

    pub fn ptr(reg: usize, offset: i128, size: usize) -> Self {
        LirVal {
            kind: LirValKind::Mem(reg, offset),
            size,
        }
    }

    pub fn reg(reg: usize, size: usize) -> Self {
        LirVal {
            kind: LirValKind::Reg(reg),
            size,
        }
    }
}

impl std::fmt::Display for LirVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LirValKind {
    Reg(usize),
    Imm(i128),
    Mem(usize, i128),
}

impl std::fmt::Display for LirValKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LirValKind::Reg(r) => f.write_fmt(format_args!("%{r}")),
            LirValKind::Imm(i) => f.write_fmt(format_args!("{i}")),
            LirValKind::Mem(r, o) => f.write_fmt(format_args!("{o}(%{r})")),
        }
    }
}
