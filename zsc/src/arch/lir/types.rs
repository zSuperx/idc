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

impl From<&Type> for LirType {
    fn from(value: &Type) -> Self {
        match value {
            Type::I8 | Type::U8 => LirType::I8,
            Type::I16 | Type::U16 => LirType::I16,
            Type::I32 | Type::U32 => LirType::I32,
            Type::I64 | Type::U64 => LirType::I64,
            Type::Pointer(id) => LirType::Ptr,
            Type::Function { args, returns } => LirType::Ptr,
            Type::Bool => LirType::I8,
            Type::Void => panic!("Can't actually compile void type"),
            Type::Unresolved(..) => panic!("Can't compile unknown type"),
            Type::Base(_) => todo!(),
        }
    }
}

impl From<TypeId> for LirType {
    fn from(value: TypeId) -> Self {
        value.lookup().into()
    }
}

impl std::fmt::Display for LirType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}").to_lowercase())
    }
}
