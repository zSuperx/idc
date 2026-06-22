mod types;
pub use types::*;

use crate::ast::*;

type Meta = ();
type NumRep = &'static str;
type Type = HirType;
type Ident = &'static str;

pub type HirExprKind = ExprKind<Meta, NumRep, Type, Ident>;
pub type HirExpr = Expr<Meta, NumRep, Type, Ident>;

pub type HirStmt = Stmt<Meta, NumRep, Type, Ident>;
pub type HirStmtKind = StmtKind<Meta, NumRep, Type, Ident>;

pub type HirObj = Obj<Meta, NumRep, Type, Ident>;
pub type HirObjKind = ObjKind<Meta, NumRep, Type, Ident>;
