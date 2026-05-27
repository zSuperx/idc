mod types;
pub use types::*;

use crate::ast::*;

type Meta = Id;
type NumRep = i128;
type Type = Id;
type Ident = &'static str;

pub type TirExprKind = ExprKind<Meta, NumRep, Ident>;
pub type TirExpr = Expr<Meta, NumRep, Ident>;

pub type TirStmt = Stmt<Meta, NumRep, Type, Ident>;
pub type TirStmtKind = StmtKind<Meta, NumRep, Type, Ident>;

pub type TirObj = Obj<Meta, NumRep, Type, Ident>;
pub type TirObjKind = ObjKind<Meta, NumRep, Type, Ident>;
