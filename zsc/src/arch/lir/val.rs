#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value {
    pub kind: ValueKind,
    /// Size of value (in bytes).
    ///
    /// This means the size that the LirVal is pointing to. In the case of LirValKind::Reg, it's the
    /// size of the value the register holds (analogous to the sizes of x86 registers). If it's a
    /// LirValKind::Mem, the size refers to the size of pointed to object
    pub size: usize,
}

impl Value {
    pub fn imm(val: i128, size: usize) -> Self {
        Value {
            kind: ValueKind::Imm(val),
            size,
        }
    }

    pub fn mem(reg: usize, size: usize) -> Self {
        Value {
            kind: ValueKind::Mem(reg),
            size,
        }
    }

    pub fn reg(reg: usize, size: usize) -> Self {
        Value {
            kind: ValueKind::Reg(reg),
            size,
        }
    }

    pub fn uninit() -> Self {
        Self {
            kind: ValueKind::Uninit,
            size: 0,
        }
    }

    pub fn is_mem(&self) -> bool {
        matches!(self.kind, ValueKind::Mem(..))
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
    Uninit,
    Reg(usize),
    Imm(i128),
    Mem(usize),
}

impl std::fmt::Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueKind::Uninit => f.write_str("N/A"),
            ValueKind::Reg(r) => f.write_fmt(format_args!("%{r}")),
            ValueKind::Imm(i) => f.write_fmt(format_args!("{i}")),
            ValueKind::Mem(r) => f.write_fmt(format_args!("ptr %{r}")),
        }
    }
}
