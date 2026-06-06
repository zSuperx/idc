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

