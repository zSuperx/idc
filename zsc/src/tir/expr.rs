use crate::ast::*;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct TirExpr {
    pub kind: TirExprKind,
    pub ty: ResolvedTypeId,
}

impl TirExpr {
    pub fn new(kind: TirExprKind, ty: ResolvedTypeId) -> Self {
        Self { kind, ty }
    }
}

#[derive(Debug, Clone)]
pub enum TirExprKind {
    Void,
    Num(i128),
    Bool(bool),
    Ident(Symbol),
    Assign {
        lhs: Box<Spanned<TirExpr>>,
        rhs: Box<Spanned<TirExpr>>,
    },
    AddrOf {
        rhs: Box<Spanned<TirExpr>>,
    },
    SizeOfTy {
        ty: Spanned<ResolvedTypeId>,
    },
    SizeOfExpr {
        expr: Box<Spanned<TirExpr>>,
    },
    Deref {
        rhs: Box<Spanned<TirExpr>>,
    },
    Index {
        expr: Box<Spanned<TirExpr>>,
        index: Box<Spanned<TirExpr>>,
    },
    Un {
        op: UnOp,
        rhs: Box<Spanned<TirExpr>>,
    },
    Bin {
        op: BinOp,
        lhs: Box<Spanned<TirExpr>>,
        rhs: Box<Spanned<TirExpr>>,
    },
    Cast {
        target_ty: Spanned<ResolvedTypeId>,
        rhs: Box<Spanned<TirExpr>>,
    },
    Call {
        callee: Box<Spanned<TirExpr>>,
        args: Vec<Spanned<TirExpr>>,
    },
}
