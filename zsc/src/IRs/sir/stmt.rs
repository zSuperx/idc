use crate::{ast::*, common::Symbol};

use super::*;

#[derive(Debug, Clone)]
pub enum SirStmt {
    Let {
        lhs: Spanned<Symbol>,
        ty: Option<Spanned<RawTypeId>>,
        rhs: Spanned<SirExpr>,
    },
    While {
        cond: Spanned<SirExpr>,
        body: Box<Spanned<SirStmt>>,
    },
    Continue,
    Break,
    If {
        cond: Spanned<SirExpr>,
        then_: Box<Spanned<SirStmt>>,
        else_: Box<Spanned<SirStmt>>,
    },
    Return(Spanned<SirExpr>),
    Block(Vec<Spanned<SirStmt>>),
    Expr(Spanned<SirExpr>),
}
