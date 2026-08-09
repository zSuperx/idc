use super::*;
use crate::ast::*;
use crate::common::*;

#[derive(Debug, Clone)]
pub enum TirObj {
    Fn {
        name: Spanned<Symbol>,
        returns: Spanned<ResolvedTypeId>,
        args: Vec<(Spanned<Symbol>, Spanned<ResolvedTypeId>)>,
        body: Box<Spanned<TirStmt>>,
    },
    Global {
        lhs: Spanned<Symbol>,
        rhs: Box<TirExprKind>,
    },
    Struct {
        name: Spanned<Symbol>,
        fields: Vec<(Spanned<Symbol>, ResolvedTypeId)>,
    },
}
