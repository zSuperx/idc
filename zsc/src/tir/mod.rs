use crate::ast::*;
use crate::prelude::*;

// Now that type checking has been performed, every AST node must have a concrete type associated
// with it, hence TypeId
type Meta = TypeId;

// Now that each AST node has a concrete type, we can finally cast numbers to their real type.
// However, we will store the value in an i128
type NumRep = i128;

// Since all types have been resolved and checked, we can replace written types in the AST with
// their IDs
type TypeType = TypeId;

// This is a very important step of this stage: Since we have done proper scope analysis, we know
// what each identifier refers to. This includes variables with the same name but different scope
type Ident = StringId;

pub type TirExprKind = ExprKind<Meta, NumRep, TypeType, Ident>;
pub type TirExpr = Expr<Meta, NumRep, TypeType, Ident>;

pub type TirStmt = Stmt<Meta, NumRep, TypeType, Ident>;
pub type TirStmtKind = StmtKind<Meta, NumRep, TypeType, Ident>;

pub type TirObj = Obj<Meta, NumRep, TypeType, Ident>;
pub type TirObjKind = ObjKind<Meta, NumRep, TypeType, Ident>;
