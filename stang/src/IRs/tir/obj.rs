use super::*;
use crate::ast::*;
use crate::common::*;

#[derive(Debug, Clone)]
pub enum TirObj {
    Fn {
        symbol: Symbol,
        returns: TypeId,
        args: Vec<(Symbol, TypeId)>,
        body: Box<TirStmt>,
    },
    Global {
        lhs: Symbol,
        rhs: Box<TirExprKind>,
    },
    Struct {
        name: Symbol,
        fields: Vec<(Symbol, TypeId)>,
    },
}
