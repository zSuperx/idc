mod types;
pub use types::*;

use crate::ast::*;
use crate::prelude::*;

type Meta = TypeId;
type NumRep = i128;
type Type = TypeId;
type Ident = StringId;

pub type TirExprKind = ExprKind<Meta, NumRep, Type, Ident>;
pub type TirExpr = Expr<Meta, NumRep, Type, Ident>;

pub type TirStmt = Stmt<Meta, NumRep, Type, Ident>;
pub type TirStmtKind = StmtKind<Meta, NumRep, Type, Ident>;

pub type TirObj = Obj<Meta, NumRep, Type, Ident>;
pub type TirObjKind = ObjKind<Meta, NumRep, Type, Ident>;
