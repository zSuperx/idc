use crate::ast::*;
use super::*;

#[derive(Debug, Clone)]
pub enum HirObj {
    Fn {
        name: Spanned<&'static str>,
        returns: Spanned<TypeId>,
        args: Vec<(Spanned<&'static str>, Spanned<TypeId>)>,
        body: Box<Spanned<HirStmt>>,
    },
    Global {
        name: Spanned<&'static str>,
        ty: Spanned<TypeId>,
        rhs: Box<Spanned<HirExpr>>,
    },
    Struct {
        name: Spanned<&'static str>,
        fields: Vec<(Spanned<&'static str>, TypeId)>,
    },
}
