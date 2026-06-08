use crate::ast::*;

#[derive(Debug, Clone)]
pub struct Expr<M, N, I> {
    pub kind: ExprKind<M, N, I>,
    pub meta: M,
}

impl<M, N, I> Expr<M, N, I> {
    pub fn new(kind: ExprKind<M, N, I>, meta: M) -> Self {
        Self { kind, meta }
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind<M, N, I> {
    Void,
    Num(N),
    Bool(bool),
    Ident(I),
    Assign {
        lhs: Box<Spanned<Expr<M, N, I>>>,
        rhs: Box<Spanned<Expr<M, N, I>>>,
    },
    AddrOf {
        rhs: Box<Spanned<Expr<M, N, I>>>,
    },
    SizeOf {
        rhs: Box<Spanned<Expr<M, N, I>>>,
    },
    Deref {
        rhs: Box<Spanned<Expr<M, N, I>>>,
    },
    Un {
        op: UnOp,
        rhs: Box<Spanned<Expr<M, N, I>>>,
    },
    Bin {
        op: BinOp,
        lhs: Box<Spanned<Expr<M, N, I>>>,
        rhs: Box<Spanned<Expr<M, N, I>>>,
    },
    Call {
        callee: Box<Spanned<Expr<M, N, I>>>,
        args: Vec<Spanned<Expr<M, N, I>>>,
    },
}

impl<M, N, I> ExprKind<M, N, I> {
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
