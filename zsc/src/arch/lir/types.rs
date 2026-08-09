use crate::ast::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LirType {
    I8,
    I16,
    I32,
    I64,
    Ptr,
}

impl LirType {
    pub fn size(&self) -> usize {
        match self {
            LirType::I8 => 1,
            LirType::I16 => 2,
            LirType::I32 => 4,
            LirType::I64 => 8,
            LirType::Ptr => 8,
        }
    }

    pub fn alignment(&self) -> usize {
        self.size()
    }
}

impl From<&ResolvedType> for LirType {
    fn from(value: &ResolvedType) -> Self {
        match value {
            ResolvedType::I8 | ResolvedType::U8 => LirType::I8,
            ResolvedType::I16 | ResolvedType::U16 => LirType::I16,
            ResolvedType::I32 | ResolvedType::U32 => LirType::I32,
            ResolvedType::I64 | ResolvedType::U64 => LirType::I64,
            ResolvedType::Pointer(id) => LirType::Ptr,
            ResolvedType::Function { args, returns } => LirType::Ptr,
            ResolvedType::Bool => LirType::I8,
            ResolvedType::Void => panic!("Can't actually compile void type"),
            ResolvedType::Unknown => panic!("Can't compile unknown type"),
            ResolvedType::Base(_) => todo!(),
        }
    }
}

impl From<ResolvedTypeId> for LirType {
    fn from(value: ResolvedTypeId) -> Self {
        value.lookup().into()
    }
}

impl std::fmt::Display for LirType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}").to_lowercase())
    }
}
