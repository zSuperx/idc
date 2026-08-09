use crate::{ast::*, common::Symbol};
use super::*;

#[derive(Debug, Clone)]
pub enum SirObj {
    Fn {
        name: Spanned<Symbol>,
        returns: Spanned<RawTypeId>,
        args: Vec<(Spanned<Symbol>, Spanned<RawTypeId>)>,
        body: Box<Spanned<SirStmt>>,
    },
    Global {
        lhs: Spanned<Symbol>,
        rhs: Box<SirExpr>,
    },
    Struct {
        name: Spanned<Symbol>,
        fields: Vec<(Spanned<Symbol>, RawTypeId)>,
    },
}
