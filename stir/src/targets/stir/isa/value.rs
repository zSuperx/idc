use super::IRType;

pub type VReg = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IRValue {
    pub kind: IRValueKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IRValueKind {
    Reg(VReg),
    Imm(i128),
    Ptr(VReg),
}

impl IRValue {
    pub fn imm(val: i128) -> Self {
        IRValue {
            kind: IRValueKind::Imm(val),
        }
    }

    pub fn from_type(reg: VReg, ty: IRType) -> Self {
        if ty.is_pointer() {
            IRValue { kind: IRValueKind::Ptr(reg) }
        } else {
            IRValue { kind: IRValueKind::Reg(reg) }
        }
    }

    pub fn ptr(reg: VReg) -> Self {
        IRValue {
            kind: IRValueKind::Ptr(reg),
        }
    }

    pub fn reg(reg: VReg) -> Self {
        IRValue {
            kind: IRValueKind::Reg(reg),
        }
    }

    pub fn is_mem(&self) -> bool {
        matches!(self.kind, IRValueKind::Ptr(..))
    }

    pub fn is_reg(&self) -> bool {
        matches!(self.kind, IRValueKind::Reg(..))
    }

    pub fn is_imm(&self) -> bool {
        matches!(self.kind, IRValueKind::Imm(..))
    }
}

impl std::fmt::Display for IRValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            IRValueKind::Reg(r) => f.write_fmt(format_args!("%{r}")),
            IRValueKind::Imm(i) => f.write_fmt(format_args!("#{i}")),
            IRValueKind::Ptr(r) => f.write_fmt(format_args!("ptr %{r}")),
        }
    }
}
