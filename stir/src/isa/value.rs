use crate::isa::IRType;

pub type VReg = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IRValue {
    pub kind: ValueKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
    Reg(VReg),
    Imm(i128),
    Ptr(VReg),
}

impl IRValue {
    pub fn imm(val: i128) -> Self {
        IRValue {
            kind: ValueKind::Imm(val),
        }
    }

    pub fn from_type(reg: VReg, ty: IRType) -> Self {
        if ty.is_pointer() {
            IRValue { kind: ValueKind::Ptr(reg) }
        } else {
            IRValue { kind: ValueKind::Reg(reg) }
        }
    }

    pub fn ptr(reg: VReg) -> Self {
        IRValue {
            kind: ValueKind::Ptr(reg),
        }
    }

    pub fn reg(reg: VReg) -> Self {
        IRValue {
            kind: ValueKind::Reg(reg),
        }
    }

    pub fn is_mem(&self) -> bool {
        matches!(self.kind, ValueKind::Ptr(..))
    }

    pub fn is_reg(&self) -> bool {
        matches!(self.kind, ValueKind::Reg(..))
    }

    pub fn is_imm(&self) -> bool {
        matches!(self.kind, ValueKind::Imm(..))
    }
}

impl std::fmt::Display for IRValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ValueKind::Reg(r) => f.write_fmt(format_args!("%{r}")),
            ValueKind::Imm(i) => f.write_fmt(format_args!("#{i}")),
            ValueKind::Ptr(r) => f.write_fmt(format_args!("ptr %{r}")),
        }
    }
}
