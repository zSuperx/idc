use crate::ast::*;

#[derive(Debug, Clone)]
pub struct Obj<M, N, T, I> {
    pub kind: ObjKind<M, N, T, I>,
    pub meta: M,
}

impl<M, N, T, I> Obj<M, N, T, I> {
    pub fn new(kind: ObjKind<M, N, T, I>, meta: M) -> Self {
        Self { kind, meta }
    }
}

#[derive(Debug, Clone)]
pub enum ObjKind<M, N, T, I> {
    Fn {
        name: Spanned<I>,
        returns: Spanned<T>,
        args: Vec<(Spanned<I>, Spanned<T>)>,
        body: Box<Spanned<Stmt<M, N, T, I>>>,
        lvars: Vec<(I, T)>,
    },
    Global {
        lhs: Spanned<I>,
        rhs: Box<Expr<M, N, I>>,
    },
    Struct {
        name: Spanned<I>,
        fields: Vec<(Spanned<I>, T)>,
    },
}
