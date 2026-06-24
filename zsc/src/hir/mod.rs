use crate::ast::*;

// Type checking has not been performed yet, so there is no meta information that needs to be stored
// with each AST node
type Meta = ();

// At this stage, we store numbers as just references to their place in the source code. 
// After type checking, we'll perform the real conversion
type NumRep = &'static str; 

// The type of the user-written types in the source code
// This refers to instances where the user types out a type name, like in:
// - let foo: T = ...
// - fn foo(bar: T) -> T { ... }
// - @T(foo)
// We'll store these as a RawType at this stage
type TypeType = RawType;

// Identifiers at this stage will just be string slices that point to the source code
type Ident = &'static str;

pub type HirExprKind = ExprKind<Meta, NumRep, TypeType, Ident>;
pub type HirExpr = Expr<Meta, NumRep, TypeType, Ident>;

pub type HirStmt = Stmt<Meta, NumRep, TypeType, Ident>;
pub type HirStmtKind = StmtKind<Meta, NumRep, TypeType, Ident>;

pub type HirObj = Obj<Meta, NumRep, TypeType, Ident>;
pub type HirObjKind = ObjKind<Meta, NumRep, TypeType, Ident>;
