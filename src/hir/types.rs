use crate::hir;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum HirType {
    Named(&'static str),
    Pointer(Box<hir::HirType>),
    Function {
        args: Vec<hir::HirType>,
        returns: Box<hir::HirType>,
    },
}
