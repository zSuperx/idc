use crate::ast::*;

#[derive(Debug, Clone)]
pub struct Expr<M, N, T, I> {
    pub kind: ExprKind<M, N, T, I>,
    pub meta: M,
}

impl<M, N, T, I> Expr<M, N, T, I> {
    pub fn new(kind: ExprKind<M, N, T, I>, meta: M) -> Self {
        Self { kind, meta }
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind<M, N, T, I> {
    Void,
    Num(N),
    Bool(bool),
    Ident(I),
    Assign {
        lhs: Box<Spanned<Expr<M, N, T, I>>>,
        rhs: Box<Spanned<Expr<M, N, T, I>>>,
    },
    AddrOf {
        rhs: Box<Spanned<Expr<M, N, T, I>>>,
    },
    SizeOf {
        rhs: Box<Spanned<Expr<M, N, T, I>>>,
    },
    Deref {
        rhs: Box<Spanned<Expr<M, N, T, I>>>,
    },
    Un {
        op: UnOp,
        rhs: Box<Spanned<Expr<M, N, T, I>>>,
    },
    Bin {
        op: BinOp,
        lhs: Box<Spanned<Expr<M, N, T, I>>>,
        rhs: Box<Spanned<Expr<M, N, T, I>>>,
    },
    Cast {
        target_ty: Spanned<T>,
        rhs: Box<Spanned<Expr<M, N, T, I>>>,
    },
    Call {
        callee: Box<Spanned<Expr<M, N, T, I>>>,
        args: Vec<Spanned<Expr<M, N, T, I>>>,
    },
}

impl<M, N, T, I> ExprKind<M, N, T, I> {
    pub fn is_valid_lvalue(&self) -> bool {
        match self {
            ExprKind::Ident(_) | ExprKind::Deref { .. } => true,
            _ => false,
        }
    }

    pub fn is_addressable(&self) -> bool {
        match self {
            ExprKind::Ident(_) => true,
            _ => false,
        }
    }
}
