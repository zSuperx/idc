use crate::ast::*;
use crate::common::*;

use super::*;

#[derive(Debug, Clone)]
pub enum TirStmt {
    Let {
        lhs: Symbol,
        ty: Option<TypeId>,
        rhs: TirExpr,
    },
    While {
        cond: TirExpr,
        body: Box<TirStmt>,
    },
    Continue,
    Break,
    If {
        cond: TirExpr,
        then_: Box<TirStmt>,
        else_: Box<TirStmt>,
    },
    Return(Option<TirExpr>),
    Block(Vec<TirStmt>),
    Expr(TirExpr),
}
