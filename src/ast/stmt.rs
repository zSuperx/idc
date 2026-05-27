use crate::ast::*;

#[derive(Debug, Clone)]
pub struct Stmt<M, N, T, I> {
    pub kind: StmtKind<M, N, T, I>,
    pub meta: M,
}

impl<M, N, T, I> Stmt<M, N, T, I> {
    pub fn new(kind: StmtKind<M, N, T, I>, meta: M) -> Self {
        Self { kind, meta }
    }
}

#[derive(Debug, Clone)]
pub enum StmtKind<M, N, T, I> {
    Let {
        lhs: Spanned<I>,
        ty: Option<Spanned<T>>,
        rhs: Spanned<Expr<M, N, I>>,
    },
    While {
        cond: Spanned<Expr<M, N, I>>,
        body: Box<Spanned<Stmt<M, N, T, I>>>,
    },
    Continue,
    Break,
    If {
        cond: Spanned<Expr<M, N, I>>,
        then_: Box<Spanned<Stmt<M, N, T, I>>>,
        else_: Box<Spanned<Stmt<M, N, T, I>>>,
    },
    Return(Option<Spanned<Expr<M, N, I>>>),
    Block(Vec<Spanned<Stmt<M, N, T, I>>>),
    Expr(Spanned<Expr<M, N, I>>),
}
