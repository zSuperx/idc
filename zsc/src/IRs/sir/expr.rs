use crate::{ast::*, common::Symbol};

#[derive(Debug, Clone)]
pub enum SirExpr {
    Void,
    Num(&'static str),
    Bool(bool),
    Ident(Symbol),
    Assign {
        lhs: Box<Spanned<SirExpr>>,
        rhs: Box<Spanned<SirExpr>>,
    },
    AddrOf {
        rhs: Box<Spanned<SirExpr>>,
    },
    SizeOfTy {
        ty: Spanned<RawTypeId>,
    },
    SizeOfExpr {
        expr: Box<Spanned<SirExpr>>,
    },
    Deref {
        rhs: Box<Spanned<SirExpr>>,
    },
    Index {
        expr: Box<Spanned<SirExpr>>,
        index: Box<Spanned<SirExpr>>,
    },
    Un {
        op: UnOp,
        rhs: Box<Spanned<SirExpr>>,
    },
    Bin {
        op: BinOp,
        lhs: Box<Spanned<SirExpr>>,
        rhs: Box<Spanned<SirExpr>>,
    },
    Cast {
        target_ty: Spanned<RawTypeId>,
        rhs: Box<Spanned<SirExpr>>,
    },
    Call {
        callee: Box<Spanned<SirExpr>>,
        args: Vec<Spanned<SirExpr>>,
    },
}
