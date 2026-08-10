use crate::IRs::{hir::*, tir::*};
use crate::aux::{SymbolInfo, SymbolKind};
use crate::common::*;
use crate::{ast::*, aux::Compiler};

impl Compiler {
    pub fn check_type(
        &mut self,
        s @ Spanned { inner: ty, span }: Spanned<TypeId>,
    ) -> Spanned<TypeId> {
        let ty = match ty.lookup() {
            Type::Unresolved(name) => {
                let Some(id) = self.type_names.get(name) else {
                    die!("Unknown type {name}: {span}")
                };
                *id
            }
            Type::Function { args, returns } => todo!(),
            Type::Pointer(id) => {
                let inner_ty = self.check_type(Spanned::new(*id, span)).inner;
                self.add_type(Type::Pointer(inner_ty))
            }
            x => return s,
        };
        Spanned::new(ty, span)
    }

    pub fn check_obj(&mut self, Spanned { inner: obj, span }: Spanned<HirObj>) -> Spanned<TirObj> {
        let inner = match obj {
            HirObj::Fn {
                name,
                returns,
                args,
                body,
                ..
            } => {
                let function_symbol = self.env.get_first(&name.inner).unwrap();
                self.func.symbol = Some(function_symbol);
                self.func.raw_name = name;

                self.env.push_scope();
                let mut checked_args = vec![];
                for (i, (argname, ty)) in args.into_iter().enumerate() {
                    let var_ty = self.check_type(ty);
                    if *var_ty.inner == Type::Void {
                        die!("Function argument cannot have type {var_ty}");
                    }

                    let argsym = self.add_local_symbol(argname, var_ty.inner, SymbolKind::Param(i));

                    checked_args.push((argsym, var_ty));
                }
                let returns = self.check_type(returns);

                let body = Box::new(self.check_stmt(*body));
                self.env.pop_scope();
                TirObj::Fn {
                    name: Spanned::new(function_symbol, span),
                    returns,
                    body,
                    args: checked_args,
                }
            }
            HirObj::Global { lhs, rhs } => todo!(),
            HirObj::Struct { name, fields } => todo!(),
        };
        Spanned::new(inner, span)
    }

    fn check_stmt(&mut self, Spanned { inner: stmt, span }: Spanned<HirStmt>) -> Spanned<TirStmt> {
        let kind = match stmt {
            HirStmt::Let { lhs, ty, rhs } => {
                let ty = ty.map(|t| self.check_type(t));
                let rhs = self.check_expr(rhs, ty.map(|t| t.inner));

                if let Some(ty) = ty
                    && ty.inner != rhs.inner.ty
                {
                    die!(
                        "Type mismatch. Expected {ty} \n...but got `{}`: {}",
                        rhs.inner.ty,
                        rhs.span,
                    );
                }
                let var_ty = rhs.inner.ty;

                let lhs_symbol = self.add_local_symbol(lhs, var_ty, SymbolKind::Local);

                TirStmt::Let {
                    lhs: lhs_symbol,
                    ty,
                    rhs,
                }
            }
            HirStmt::While { cond, body } => todo!(),
            HirStmt::Continue => todo!(),
            HirStmt::Break => todo!(),
            HirStmt::If { cond, then_, else_ } => {
                let cond = self.check_expr(cond, None);
                let cond_ty = cond.inner.ty;
                if *cond_ty != Type::Bool {
                    die!(
                        "Type mismatch. Expected `{}` but got `{}`: {}",
                        Type::Bool,
                        cond_ty,
                        cond.span
                    )
                }
                let then_ = Box::new(self.check_stmt(*then_));
                let else_ = Box::new(self.check_stmt(*else_));
                TirStmt::If { cond, then_, else_ }
            }
            HirStmt::Return(val) => {
                let Type::Function { returns, .. } =
                    *self.lookup_symbol(self.current_function()).ty
                else {
                    panic!("Function with non-function type");
                };
                let checked_val = self.check_expr(val, Some(returns));
                if checked_val.inner.ty != returns {
                    die!(
                        "Mismatched return type. Function {} expects {returns} but got {}: {}",
                        self.func.raw_name,
                        checked_val.inner.ty,
                        checked_val.span
                    )
                }
                TirStmt::Return(checked_val)
            }
            HirStmt::Block(s) => {
                self.env.push_scope();
                let stmt = TirStmt::Block(s.into_iter().map(|st| self.check_stmt(st)).collect());
                self.env.pop_scope();
                stmt
            }
            HirStmt::Expr(e) => {
                let e = self.check_expr(e, None);
                TirStmt::Expr(e)
            }
        };
        // Inherit the same span from the previous phase
        Spanned::new(kind, span)
    }

    fn check_expr(
        &mut self,
        Spanned { inner: expr, span }: Spanned<HirExpr>,
        hint: Option<TypeId>,
    ) -> Spanned<TirExpr> {
        let inner = match expr {
            HirExpr::Void => {
                let kind = TirExprKind::Void;
                TirExpr::new(kind, self.add_type(Type::Void))
            }
            HirExpr::Num(int_str) => {
                let ty = match hint {
                    Some(hint_id) => {
                        let hint_ty = hint_id;
                        if hint_ty.is_integral() {
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
                let kind = TirExprKind::Bool(b);
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
                let checked_lhs = self.check_expr(*lhs, hint);
                let lhs_ty = checked_lhs.inner.ty;
                let checked_rhs = self.check_expr(*rhs, Some(lhs_ty));
                let rhs_ty = checked_rhs.inner.ty;
                if lhs_ty != rhs_ty {
                    die!("Cannot assign a `{rhs_ty}` to `{lhs_ty}`: {span}")
                }
                // TODO: add a new pass for this
                if !checked_lhs.inner.is_valid_lvalue() {
                    die!(
                        "Cannot assign to this expression as it is not a valid LVALUE: {}",
                        checked_lhs.span,
                    )
                }
                let ty = lhs_ty;
                let kind = TirExprKind::Assign {
                    lhs: Box::new(checked_lhs),
                    rhs: Box::new(checked_rhs),
                };
                TirExpr::new(kind, ty)
            }
            HirExpr::Index { expr, index } => {
                // x[i] => *(x + i * sizeof(*x))
                // TODO: change this to lower indexing manually. Don't convert it to just dererf
                // arithmetic, a lot of information ends up getting lost
                // Instead, change the LirVal::Mem API to include base reg, offset reg, scale imm
                // and displacement imm
                // This is in hopes of emitting a physical instruction like [rbx + rax * 8 + 1]
                let expr = self.check_expr(*expr, None);
                let u64_ty = self.add_type(Type::U64);
                let index = self.check_expr(*index, Some(u64_ty));
                let index_ty = index.inner.ty;
                if *index_ty != Type::U64 {
                    die!("Cannot index a pointer with a {index_ty}. Use a u64 instead: {index}")
                }
                let Type::Pointer(elem_ty) = *expr.inner.ty else {
                    die!("Cannot index a non-pointer type {}: {expr}", expr.inner.ty)
                };
                let span = expr.span;
                let sizeof = Spanned::new(
                    TirExpr::new(TirExprKind::Num(elem_ty.size() as i128), u64_ty),
                    span,
                );
                let mul = Spanned::new(
                    TirExpr::new(
                        TirExprKind::Bin {
                            op: BinOp::Mul,
                            lhs: Box::new(index),
                            rhs: Box::new(sizeof),
                        },
                        u64_ty,
                    ),
                    span,
                );
                let cast_ptr_to_u64 = Spanned::new(
                    TirExpr::new(
                        TirExprKind::Cast {
                            target_ty: Spanned::new(u64_ty, span),
                            rhs: Box::new(expr),
                        },
                        u64_ty,
                    ),
                    span,
                );
                let add = Spanned::new(
                    TirExpr::new(
                        TirExprKind::Bin {
                            op: BinOp::Add,
                            lhs: Box::new(cast_ptr_to_u64),
                            rhs: Box::new(mul),
                        },
                        u64_ty,
                    ),
                    span,
                );
                let cast_to_elem_ty = Spanned::new(
                    TirExpr::new(
                        TirExprKind::Cast {
                            target_ty: Spanned::new(elem_ty, span),
                            rhs: Box::new(add.clone()),
                        },
                        elem_ty,
                    ),
                    span,
                );
                let kind = TirExprKind::Deref { rhs: Box::new(add) };
                TirExpr::new(kind, elem_ty)
            }
            HirExpr::Deref { rhs } => {
                // If we get a hint of *T, the sub-expr should be checked with hint T
                let hint_inner = hint.and_then(|h| match h.lookup() {
                    Type::Pointer(id) => Some(*id),
                    _ => None,
                });
                // We don't want to check the inner expression yet. First check its kind
                match rhs.inner {
                    // If we're dereferencing an addrof, they cancel out
                    // e.g. (*&y == y). In this case, pull out the AddrOf's sub-expr
                    // and check it separately. We do this to avoid the AddrOf book-keeping code
                    HirExpr::AddrOf { rhs } => self.check_expr(*rhs, hint_inner).inner,
                    _ => {
                        let checked_rhs = Box::new(self.check_expr(*rhs, hint_inner));
                        let checked_ty = checked_rhs.inner.ty;
                        // A deref can only happen on a pointer, and its type will be whatever
                        // the pointer is pointing to
                        let ty = match checked_ty.lookup() {
                            Type::Pointer(id) => *id,
                            _ => die!(
                                "Cannot dereference non-pointer type `{checked_ty}`: {checked_rhs}"
                            ),
                        };
                        let kind = TirExprKind::Deref { rhs: checked_rhs };
                        TirExpr::new(kind, ty)
                    }
                }
            }
            HirExpr::AddrOf { rhs } => {
                // We don't want to check the inner expression yet. First check its kind
                match rhs.inner {
                    // AddrOf is kind of a ty-expression. It can ONLY operate on
                    // named values. The exception is AddrOf(Deref(...)), but...
                    HirExpr::Ident(..) => {
                        // If Ident, perform the official check
                        let checked_rhs = Box::new(self.check_expr(*rhs, hint));
                        let rhs_ty = checked_rhs.inner.ty;
                        let TirExprKind::Ident(var_symbol) = checked_rhs.inner.kind else {
                            unreachable!("rhs was shown to be an Ident")
                        };

                        // Given &x, where x: T, &x has type *T
                        let ty = self.add_type(Type::Pointer(rhs_ty));

                        // Mark this symbol as address-taken
                        let info = self.lookup_symbol_mut(var_symbol);
                        info.address_taken = true;

                        let kind = TirExprKind::AddrOf { rhs: checked_rhs };
                        TirExpr::new(kind, ty)
                    }
                    // ... if we're taking address of a dereference, they cancel out e.g. (&*y == y)
                    HirExpr::Deref { rhs } => self.check_expr(*rhs, hint).inner,
                    _ => die!("Cannot take the address of this type of expression: {rhs}"),
                }
            }
            HirExpr::Cast { target_ty, rhs } => {
                // TODO: enforce type casting rules
                // Casting should be valid between:
                // - Same sized types (this means all pointers can be cast to and from each other)
                // - Any primitive with any other primitive
                let checked_ty = self.check_type(target_ty);
                let checked_rhs = Box::new(self.check_expr(*rhs, Some(checked_ty.inner)));
                let ty = checked_ty.inner;
                let kind = TirExprKind::Cast {
                    target_ty: checked_ty,
                    rhs: checked_rhs,
                };
                TirExpr::new(kind, ty)
            }
            HirExpr::Un { op, rhs } => {
                let checked_rhs = self.check_expr(*rhs, hint);
                let rhs_ty = checked_rhs.inner.ty;
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
                    op,
                    rhs: Box::new(checked_rhs),
                };
                TirExpr::new(kind, ty)
            }
            HirExpr::Bin { op, lhs, rhs } => {
                let checked_lhs = self.check_expr(*lhs, hint);
                let lhs_ty = checked_lhs.inner.ty;
                let checked_rhs = self.check_expr(*rhs, Some(lhs_ty));
                let rhs_ty = checked_rhs.inner.ty;
                let ty = match op {
                    BinOp::Add => {
                        if (lhs_ty != rhs_ty) || !lhs_ty.is_integral() {
                            die!("Cannot add `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Sub => {
                        if (lhs_ty != rhs_ty) || !lhs_ty.is_integral() {
                            die!("Cannot subtract `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Mul => {
                        if (lhs_ty != rhs_ty) || !lhs_ty.is_integral() {
                            die!("Cannot multiply `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Div => {
                        if (lhs_ty != rhs_ty) || !(lhs_ty).is_integral() {
                            die!("Cannot divide `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Eq => {
                        if lhs_ty != rhs_ty {
                            die!("Cannot compare `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        self.add_type(Type::Bool)
                    }
                    BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                        if (lhs_ty != rhs_ty) || !(lhs_ty).is_integral() {
                            die!("Cannot compare `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        self.add_type(Type::Bool)
                    }
                };
                let kind = TirExprKind::Bin {
                    op,
                    lhs: Box::new(checked_lhs),
                    rhs: Box::new(checked_rhs),
                };
                TirExpr::new(kind, ty)
            }
            HirExpr::Call { callee, args } => {
                // TODO: we can't check if the user is passing in the right types of arguments until
                // we add the callee to the global symbol table
                let callee = Box::new(self.check_expr(*callee, hint));
                let Type::Function { returns, .. } = *callee.inner.ty else {
                    die!("Function callee does not resolve to a function type: {callee}");
                };

                let args = args.into_iter().map(|a| self.check_expr(a, None)).collect();
                let kind = TirExprKind::Call { callee, args };
                TirExpr::new(kind, returns)
            }
            HirExpr::SizeOfTy { ty } => {
                let ty_size = self.check_type(ty).inner.size();
                let kind = TirExprKind::Num(ty_size as i128);
                let ty = self.add_type(Type::U64);
                TirExpr::new(kind, ty)
            }
            HirExpr::SizeOfExpr { expr } => {
                let ty_size = self.check_expr(*expr, None).inner.ty.size();
                let kind = TirExprKind::Num(ty_size as i128);
                let ty = self.add_type(Type::U64);
                TirExpr::new(kind, ty)
            }
        };
        Spanned::new(inner, span)
    }
}
