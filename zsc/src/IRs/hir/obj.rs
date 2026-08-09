use crate::ast::*;
use super::*;

#[derive(Debug, Clone)]
pub enum HirObj {
    Fn {
        name: Spanned<&'static str>,
        returns: Spanned<RawTypeId>,
        args: Vec<(Spanned<&'static str>, Spanned<RawTypeId>)>,
        body: Box<Spanned<HirStmt>>,
    },
    Global {
        lhs: Spanned<&'static str>,
        rhs: Box<HirExpr>,
    },
    Struct {
        name: Spanned<&'static str>,
        fields: Vec<(Spanned<&'static str>, RawTypeId)>,
    },
}
