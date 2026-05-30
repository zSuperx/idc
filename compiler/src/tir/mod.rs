mod types;
use registry::Id;
pub use types::*;

use crate::ast::*;

type Meta = TypeId;
type NumRep = i128;
type Type = TypeId;
type Ident = VarId;

pub type VarId = Id<String>;

pub type TirExprKind = ExprKind<Meta, NumRep, Ident>;
pub type TirExpr = Expr<Meta, NumRep, Ident>;

pub type TirStmt = Stmt<Meta, NumRep, Type, Ident>;
pub type TirStmtKind = StmtKind<Meta, NumRep, Type, Ident>;

pub type TirObj = Obj<Meta, NumRep, Type, Ident>;
pub type TirObjKind = ObjKind<Meta, NumRep, Type, Ident>;
