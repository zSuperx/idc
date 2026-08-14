#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value {
    pub kind: ValueKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
    Reg(usize),
    Imm(i128),
    Mem(usize),
    Arg(usize),
}

impl Value {
    pub fn imm(val: i128) -> Self {
        Value {
            kind: ValueKind::Imm(val),
        }
    }

    pub fn mem(reg: usize) -> Self {
        Value {
            kind: ValueKind::Mem(reg),
        }
    }

    pub fn arg(arg: usize) -> Self {
        Value {
            kind: ValueKind::Arg(arg),
        }
    }

    pub fn reg(reg: usize) -> Self {
        Value {
            kind: ValueKind::Reg(reg),
        }
    }

    pub fn is_mem(&self) -> bool {
        matches!(self.kind, ValueKind::Mem(..))
    }

    pub fn is_reg(&self) -> bool {
        matches!(self.kind, ValueKind::Reg(..))
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ValueKind::Reg(r) => f.write_fmt(format_args!("%{r}")),
            ValueKind::Imm(i) => f.write_fmt(format_args!("{i}")),
            ValueKind::Mem(r) => f.write_fmt(format_args!("ptr %{r}")),
            ValueKind::Arg(i) => f.write_fmt(format_args!("args[{i}]")),
        }
    }
}
