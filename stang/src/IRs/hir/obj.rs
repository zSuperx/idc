use super::*;
use crate::ast::*;

#[derive(Debug, Clone)]
pub enum HirObj {
    Fn(ParsedFunction),
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

#[derive(Debug, Clone)]
pub struct ParsedFunction {
    pub name: Spanned<&'static str>,
    pub returns: Spanned<TypeId>,
    pub args: Vec<(Spanned<&'static str>, Spanned<TypeId>)>,
    pub body: Box<Spanned<HirStmt>>,
}
