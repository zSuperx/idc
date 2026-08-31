use super::IRType;

pub type VReg = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IRValue {
    Reg(VReg),
    Imm(i128),
    Ptr(VReg),
    Arg(usize),
}

impl IRValue {
    pub fn from_type(reg: VReg, ty: IRType) -> Self {
        if ty.is_pointer() {
            IRValue::Ptr(reg)
        } else {
            IRValue::Reg(reg)
        }
    }

    pub fn is_mem(&self) -> bool {
        matches!(self, IRValue::Ptr(..))
    }

    pub fn is_reg(&self) -> bool {
        matches!(self, IRValue::Reg(..))
    }

    pub fn is_imm(&self) -> bool {
        matches!(self, IRValue::Imm(..))
    }
}

impl std::fmt::Display for IRValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IRValue::Arg(n) => f.write_fmt(format_args!("%arg.{n}")),
            IRValue::Reg(r) => f.write_fmt(format_args!("%{r}")),
            IRValue::Imm(i) => f.write_fmt(format_args!("#{i}")),
            IRValue::Ptr(r) => f.write_fmt(format_args!("ptr %{r}")),
        }
    }
}
