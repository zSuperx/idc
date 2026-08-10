use crate::ast::*;

#[derive(Debug, Clone)]
pub enum HirExpr {
    Void,
    Num(&'static str),
    Bool(bool),
    Ident(&'static str),
    Assign {
        lhs: Box<Spanned<HirExpr>>,
        rhs: Box<Spanned<HirExpr>>,
    },
    AddrOf {
        rhs: Box<Spanned<HirExpr>>,
    },
    SizeOfTy {
        ty: Spanned<TypeId>,
    },
    SizeOfExpr {
        expr: Box<Spanned<HirExpr>>,
    },
    Deref {
        rhs: Box<Spanned<HirExpr>>,
    },
    Index {
        expr: Box<Spanned<HirExpr>>,
        index: Box<Spanned<HirExpr>>,
    },
    Un {
        op: UnOp,
        rhs: Box<Spanned<HirExpr>>,
    },
    Bin {
        op: BinOp,
        lhs: Box<Spanned<HirExpr>>,
        rhs: Box<Spanned<HirExpr>>,
    },
    Cast {
        target_ty: Spanned<TypeId>,
        rhs: Box<Spanned<HirExpr>>,
    },
    Call {
        callee: Box<Spanned<HirExpr>>,
        args: Vec<Spanned<HirExpr>>,
    },
}
