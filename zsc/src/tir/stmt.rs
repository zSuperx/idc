use crate::ast::*;
use crate::prelude::*;

use super::*;

#[derive(Debug, Clone)]
pub enum TirStmt {
    Let {
        lhs: Spanned<Symbol>,
        ty: Option<Spanned<ResolvedTypeId>>,
        rhs: Spanned<TirExpr>,
    },
    While {
        cond: Spanned<TirExpr>,
        body: Box<Spanned<TirStmt>>,
    },
    Continue,
    Break,
    If {
        cond: Spanned<TirExpr>,
        then_: Box<Spanned<TirStmt>>,
        else_: Box<Spanned<TirStmt>>,
    },
    Return(Spanned<TirExpr>),
    Block(Vec<Spanned<TirStmt>>),
    Expr(Spanned<TirExpr>),
}
