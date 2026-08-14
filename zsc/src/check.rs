use crate::IRs::{hir::*, tir::*};
use crate::aux::{SymbolInfo, SymbolKind};
use crate::common::*;
use crate::{ast::*, aux::Compiler};

impl Compiler {
    pub fn check_type(&mut self, s @ Spanned { inner: ty, span }: &Spanned<TypeId>) -> TypeId {
        let ty = match ty.lookup() {
            Type::Unresolved(name) => {
                let Some(id) = self.type_names.get(name) else {
                    die!("Unknown type {name}: {span}")
                };
                *id
            }
            Type::Function { args, returns } => todo!(),
            Type::Pointer(id) => {
                let inner_ty = self.check_type(&Spanned::new(*id, *span));
                self.add_type(Type::Pointer(inner_ty))
            }
            _ => return s.inner,
        };
        ty
    }

    pub fn check_obj(&mut self, Spanned { inner: obj, span }: Spanned<HirObj>) -> TirObj {
        let inner = match obj {
            HirObj::Fn {
                name,
                returns,
                args,
                body,
                ..
            } => {
                self.current_function = self.env.get_first(&name.inner);
                self.functions
                    .insert(self.current_function.unwrap(), Default::default());
                self.env.push_scope();
                let mut checked_args = vec![];
                for (i, (argname, ty)) in args.into_iter().enumerate() {
                    let var_ty = self.check_type(&ty);
                    if *var_ty == Type::Void {
                        die!("Function argument cannot have type {var_ty}");
                    }

                    let argsym = self.add_local_symbol(argname, var_ty, SymbolKind::Arg(i));

                    checked_args.push((argsym, var_ty));
                }
                let returns = self.check_type(&returns);

                let body = Box::new(self.check_stmt(*body));
                self.env.pop_scope();
                TirObj::Fn {
                    name: self.current_function.unwrap(),
                    returns,
                    body,
                    args: checked_args,
                }
            }
            HirObj::Global { name, ty, rhs } => todo!(),
            HirObj::Struct { name, fields } => todo!(),
        };
        inner
    }

    fn check_stmt(&mut self, Spanned { inner: stmt, span }: Spanned<HirStmt>) -> TirStmt {
        let kind = match stmt {
            HirStmt::Let { lhs, ty, rhs } => {
                let ty = ty.map(|t| self.check_type(&t));
                let checked_rhs = self.check_expr(&rhs, ty.map(|t| t));

                if let Some(ty) = ty
                    && ty != checked_rhs.ty
                {
                    die!(
                        "Type mismatch. Expected {ty} but got `{}`: {}",
                        checked_rhs.ty,
                        span,
                    );
                }
                let var_ty = checked_rhs.ty;

                let lhs_symbol = self.add_local_symbol(lhs, var_ty, SymbolKind::Local);

                TirStmt::Let {
                    lhs: lhs_symbol,
                    ty,
                    rhs: checked_rhs,
                }
            }
            HirStmt::While { cond, body } => {
                let checked_cond = self.check_expr(&cond, None);
                let cond_ty = checked_cond.ty;
                if *cond_ty != Type::Bool {
                    die!(
                        "Type mismatch. Expected `{}` but got `{}`: {}",
                        Type::Bool,
                        cond_ty,
                        span
                    )
                }
                self.env.push_scope();
                self.loop_depth += 1;
                let checked_body = self.check_stmt(*body);
                self.loop_depth -= 1;
                self.env.push_scope();
                TirStmt::While {
                    cond: checked_cond,
                    body: Box::new(checked_body),
                }
            }
            HirStmt::Continue => {
                if self.loop_depth > 0 {
                    TirStmt::Continue
                } else {
                    die!("Continue statements can only be used inside a loop body: {span}");
                }
            }
            HirStmt::Break => {
                if self.loop_depth > 0 {
                    TirStmt::Break
                } else {
                    die!("Break statements can only be used inside a loop body: {span}");
                }
            }
            HirStmt::If { cond, then_, else_ } => {
                let checked_cond = self.check_expr(&cond, None);
                let cond_ty = checked_cond.ty;
                if *cond_ty != Type::Bool {
                    die!(
                        "Type mismatch. Expected `{}` but got `{}`: {}",
                        Type::Bool,
                        cond_ty,
                        span
                    )
                }
                let checked_then = Box::new(self.check_stmt(*then_));
                let checked_else = Box::new(self.check_stmt(*else_));
                TirStmt::If {
                    cond: checked_cond,
                    then_: checked_then,
                    else_: checked_else,
                }
            }
            HirStmt::Return(val) => {
                let Type::Function { returns, .. } =
                    *self.lookup_symbol(self.current_function.unwrap()).ty
                else {
                    panic!("Function with non-function type");
                };

                let checked_val = self.check_expr(&val, Some(returns));
                if checked_val.ty != returns {
                    die!(
                        "Mismatched return type. Function expects {returns} but got {}: {} {}",
                        checked_val.ty,
                        self.get_current_function_info().raw_name,
                        span
                    )
                }

                if *returns == Type::Void {
                    TirStmt::Return(None)
                } else {
                    TirStmt::Return(Some(checked_val))
                }
            }
            HirStmt::Block(s) => {
                self.env.push_scope();
                let stmt = TirStmt::Block(s.into_iter().map(|st| self.check_stmt(st)).collect());
                self.env.pop_scope();
                stmt
            }
            HirStmt::Expr(e) => {
                let e = self.check_expr(&e, None);
                TirStmt::Expr(e)
            }
        };

        kind
    }

    fn check_expr(
        &mut self,
        Spanned { inner: expr, span }: &Spanned<HirExpr>,
        hint: Option<TypeId>,
    ) -> TirExpr {
        let inner = match expr {
            HirExpr::Void => {
                let kind = TirExprKind::Void;
                TirExpr::new(kind, self.add_type(Type::Void))
            }
            HirExpr::Num(int_str) => {
                let ty = match hint {
                    Some(hint_id) => {
                        if hint_id.is_integral() {
                            hint_id
                        } else {
                            self.add_type(Type::I32)
                        }
                    }
                    None => self.add_type(Type::I32),
                };
                let result = match ty.lookup() {
                    Type::I8 => int_str.parse::<i8>().map(|i| i as i128),
                    Type::U8 => int_str.parse::<u8>().map(|i| i as i128),
                    Type::I16 => int_str.parse::<i16>().map(|i| i as i128),
                    Type::U16 => int_str.parse::<u16>().map(|i| i as i128),
                    Type::I32 => int_str.parse::<i32>().map(|i| i as i128),
                    Type::U32 => int_str.parse::<u32>().map(|i| i as i128),
                    Type::I64 => int_str.parse::<i64>().map(|i| i as i128),
                    Type::U64 => int_str.parse::<u64>().map(|i| i as i128),
                    _ => {
                        die!("`{int_str}` could not be parsed as a {ty}");
                    }
                };
                let Ok(num) = result else {
                    die!("`{int_str}` could not be parsed as a `{ty}`: {span}");
                };
                let kind = TirExprKind::Num(num);
                TirExpr::new(kind, ty)
            }
            HirExpr::Bool(b) => {
                let ty = self.add_type(Type::Bool);
                let kind = TirExprKind::Bool(*b);
                TirExpr::new(kind, ty)
            }
            HirExpr::Ident(i) => {
                let Some(symbol) = self.env.get(&i) else {
                    die!("Undefined variable {i}: {span}");
                };
                let SymbolInfo { ty, .. } = self.lookup_symbol(symbol);
                let kind = TirExprKind::Ident(symbol);
                TirExpr::new(kind, *ty)
            }
            HirExpr::Assign { lhs, rhs } => {
                let checked_lhs = self.check_expr(lhs, hint);
                let lhs_ty = checked_lhs.ty;
                let checked_rhs = self.check_expr(rhs, Some(lhs_ty));
                let rhs_ty = checked_rhs.ty;
                if lhs_ty != rhs_ty {
                    die!("Cannot assign a `{rhs_ty}` to `{lhs_ty}`: {span}")
                }
                // TODO: add a new pass for this
                if !checked_lhs.is_valid_lvalue() {
                    die!(
                        "Cannot assign to this expression as it is not a valid LVALUE: {}",
                        span,
                    )
                }
                let ty = lhs_ty;
                let kind = TirExprKind::Assign {
                    lhs: Box::new(checked_lhs),
                    rhs: Box::new(checked_rhs),
                };
                TirExpr::new(kind, ty)
            }
            HirExpr::Index { base, index } => {
                // If pointer arithmetic is set in place, index can just become:
                // x[i] => *(x + i)
                let checked_expr = self.check_expr(base, None);
                let hint_ty = self.add_type(Type::U64);
                let checked_index = self.check_expr(index, Some(hint_ty));
                if !checked_expr.ty.is_pointer() {
                    die!("Can't index into a {} type: {span}", checked_index.ty);
                }
                if !checked_index.ty.is_integral() {
                    die!("Can't index using a {} type: {span}", checked_index.ty);
                }
                let deref_target = self.scale_ptr_int_math(checked_expr, BinOp::Add, checked_index);
                let ty = deref_target.ty.get_pointee();
                let kind = TirExprKind::Deref {
                    target: Box::new(deref_target),
                };
                TirExpr::new(kind, ty)
            }
            HirExpr::Deref { rhs } => {
                // If we get a hint of *T, the sub-expr should be checked with hint T
                let hint_inner = hint.and_then(|h| match h.lookup() {
                    Type::Pointer(id) => Some(*id),
                    _ => None,
                });
                // We don't want to check the inner expression yet. First check its kind
                match &rhs.inner {
                    // If we're dereferencing an addrof, they cancel out
                    // e.g. (*&y == y). In this case, pull out the AddrOf's sub-expr
                    // and check it separately. We do this to avoid the AddrOf book-keeping code
                    HirExpr::AddrOf { rhs } => self.check_expr(&rhs, hint_inner),
                    _ => {
                        let checked_rhs = Box::new(self.check_expr(rhs, hint_inner));
                        let checked_ty = checked_rhs.ty;
                        // A deref can only happen on a pointer, and its type will be whatever
                        // the pointer is pointing to
                        let ty = match checked_ty.lookup() {
                            Type::Pointer(id) => *id,
                            _ => die!("Cannot dereference non-pointer type {span}"),
                        };
                        let kind = TirExprKind::Deref { target: checked_rhs };
                        TirExpr::new(kind, ty)
                    }
                }
            }
            HirExpr::AddrOf { rhs } => {
                // We don't want to check the inner expression yet. First check its kind
                match &rhs.inner {
                    // AddrOf is kind of a ty-expression. It can ONLY operate on
                    // named values. The exception is AddrOf(Deref(...)), but...
                    HirExpr::Ident(..) => {
                        // If Ident, perform the official check
                        let checked_rhs = Box::new(self.check_expr(rhs, hint));
                        let rhs_ty = checked_rhs.ty;
                        let TirExprKind::Ident(var_symbol) = checked_rhs.kind else {
                            unreachable!("rhs was shown to be an Ident")
                        };

                        // Given &x, where x: T, &x has type *T
                        let ty = self.add_type(Type::Pointer(rhs_ty));

                        // Mark this symbol as address-taken
                        let info = self.lookup_symbol_mut(var_symbol);
                        info.address_taken = true;

                        let kind = TirExprKind::AddrOf { expr: checked_rhs };
                        TirExpr::new(kind, ty)
                    }
                    // ... if we're taking address of a dereference, they cancel out e.g. (&*y == y)
                    HirExpr::Deref { rhs } => self.check_expr(&rhs, hint),
                    _ => die!("Cannot take the address of this type of expression: {rhs}"),
                }
            }
            HirExpr::Cast { target_ty, rhs } => {
                // TODO: enforce type casting rules
                // Casting should be valid between:
                // - Same sized types (this means all pointers can be cast to and from each other)
                // - Any primitive with any other primitive
                let checked_ty = self.check_type(target_ty);
                let checked_rhs = Box::new(self.check_expr(rhs, Some(checked_ty)));
                let ty = checked_ty;
                let kind = TirExprKind::Cast {
                    target_ty: checked_ty,
                    expr: checked_rhs,
                };
                TirExpr::new(kind, ty)
            }
            HirExpr::Un { op, rhs } => {
                let checked_rhs = self.check_expr(rhs, hint);
                let rhs_ty = checked_rhs.ty;
                let ty = match op {
                    UnOp::Not => {
                        if *rhs_ty != Type::Bool {
                            die!("Cannot logical not a `{rhs_ty}`: {span}")
                        }
                        rhs_ty
                    }
                    UnOp::Neg => {
                        if !rhs_ty.is_signed() {
                            die!("Cannot negate a `{rhs_ty}`: {span}")
                        }
                        rhs_ty
                    }
                };
                let kind = TirExprKind::Un {
                    op: *op,
                    rhs: Box::new(checked_rhs),
                };
                TirExpr::new(kind, ty)
            }
            HirExpr::Bin { op, lhs, rhs } => {
                let checked_lhs = self.check_expr(lhs, hint);
                let lhs_ty = checked_lhs.ty;
                let checked_rhs = self.check_expr(rhs, Some(lhs_ty));
                let rhs_ty = checked_rhs.ty;

                match (checked_lhs, op, checked_rhs) {
                    (ptr, BinOp::Sub | BinOp::Add, int) | (int, BinOp::Add, ptr)
                        if ptr.ty.is_pointer() && int.ty.is_integral() =>
                    {
                        self.scale_ptr_int_math(ptr, *op, int)
                    }
                    (ptr1, BinOp::Sub, ptr2) if ptr1.ty.is_pointer() && ptr2.ty == ptr1.ty => {
                        let ty = self.add_type(Type::U64);
                        let kind = TirExprKind::Bin {
                            op: *op,
                            lhs: Box::new(ptr1),
                            rhs: Box::new(ptr2),
                        };
                        TirExpr::new(kind, ty)
                    }
                    (int1, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div, int2)
                        if int1.ty.is_integral() && int2.ty == int1.ty =>
                    {
                        let ty = int1.ty;
                        let kind = TirExprKind::Bin {
                            op: *op,
                            lhs: Box::new(int1),
                            rhs: Box::new(int2),
                        };
                        TirExpr::new(kind, ty)
                    }
                    (ptr1, BinOp::Le | BinOp::Lt | BinOp::Ge | BinOp::Gt, ptr2)
                        if ptr1.ty.is_pointer() && ptr2.ty == ptr1.ty =>
                    {
                        let ty = self.add_type(Type::Bool);
                        let kind = TirExprKind::Bin {
                            op: *op,
                            lhs: Box::new(ptr1),
                            rhs: Box::new(ptr2),
                        };
                        TirExpr::new(kind, ty)
                    }
                    (int1, BinOp::Le | BinOp::Lt | BinOp::Ge | BinOp::Gt, int2)
                        if int1.ty.is_integral() && int2.ty == int1.ty =>
                    {
                        let ty = self.add_type(Type::Bool);
                        let kind = TirExprKind::Bin {
                            op: *op,
                            lhs: Box::new(int1),
                            rhs: Box::new(int2),
                        };
                        TirExpr::new(kind, ty)
                    }
                    (any1, BinOp::Eq | BinOp::Ne, any2) if any1.ty == any2.ty => {
                        let ty = self.add_type(Type::Bool);
                        let kind = TirExprKind::Bin {
                            op: *op,
                            lhs: Box::new(any1),
                            rhs: Box::new(any2),
                        };
                        TirExpr::new(kind, ty)
                    }
                    _ => die!("Incompatible operation between {lhs_ty} and {rhs_ty}: {span}"),
                }
            }
            HirExpr::Call { callee, args } => {
                // TODO: we can't check if the user is passing in the right types of arguments until
                // we add the callee to the global symbol table
                let callee = Box::new(self.check_expr(callee, hint));
                let Type::Function { returns, .. } = *callee.ty else {
                    die!("Function callee does not resolve to a function type: {span}");
                };

                let args = args.into_iter().map(|a| self.check_expr(a, None)).collect();
                let kind = TirExprKind::Call { callee, args };
                TirExpr::new(kind, returns)
            }
            HirExpr::SizeOfTy { ty } => {
                let ty_size = self.check_type(ty).bytes();
                let kind = TirExprKind::Num(ty_size as i128);
                let ty = self.add_type(Type::U64);
                TirExpr::new(kind, ty)
            }
            HirExpr::SizeOfExpr { expr } => {
                let ty_size = self.check_expr(expr, None).ty.bytes();
                let kind = TirExprKind::Num(ty_size as i128);
                let ty = self.add_type(Type::U64);
                TirExpr::new(kind, ty)
            }
        };
        inner
    }

    fn scale_ptr_int_math(&mut self, ptr: TirExpr, op: BinOp, int: TirExpr) -> TirExpr {
        let ptr_ty = ptr.ty;
        let scaled_int = {
            let scale = {
                let pointee_size = ptr.ty.get_pointee().bytes();
                let kind = TirExprKind::Num(pointee_size as i128);
                TirExpr::new(kind, self.add_type(Type::U64))
            };
            let kind = TirExprKind::Bin {
                op: BinOp::Mul,
                lhs: Box::new(int),
                rhs: Box::new(scale),
            };
            TirExpr::new(kind, self.add_type(Type::U64))
        };
        let kind = TirExprKind::Bin {
            op,
            lhs: Box::new(ptr),
            rhs: Box::new(scaled_int),
        };
        TirExpr::new(kind, ptr_ty)
    }
}
