use crate::ast::*;
use crate::common::*;

#[derive(Debug, Clone)]
pub struct TirExpr {
    pub kind: TirExprKind,
    pub ty: TypeId,
}

impl TirExpr {
    pub fn new(kind: TirExprKind, ty: TypeId) -> Self {
        Self { kind, ty }
    }

    pub fn is_valid_lvalue(&self) -> bool {
        match self.kind {
            TirExprKind::ValueOf(..) | TirExprKind::Load { .. } => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TirExprKind {
    Void,
    Num(i128),
    Bool(bool),
    Store {
        ptr: Box<TirExpr>,
        val: Box<TirExpr>,
    },
    Load {
        inner: Box<TirExpr>,
    },
    ValueOf(Symbol),
    AddrOf(Symbol),
    Un {
        op: UnOp,
        rhs: Box<TirExpr>,
    },
    Bin {
        op: BinOp,
        lhs: Box<TirExpr>,
        rhs: Box<TirExpr>,
    },
    Cast {
        target_ty: TypeId,
        expr: Box<TirExpr>,
    },
    Call {
        callee: Box<TirExpr>,
        args: Vec<TirExpr>,
    },
}
