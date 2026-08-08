use crate::aux::{SymbolInfo, SymbolKind};
use crate::hir::*;
use crate::prelude::*;
use crate::tir::*;
use crate::{ast::*, aux::Compiler};

impl Compiler {
    fn check_type(&mut self, ty: Spanned<RawTypeId>) -> Spanned<ResolvedTypeId> {
        let id = match ty.inner.lookup() {
            RawType::Base(name) => {
                let tir_ty = match self.builtin_types.get(name) {
                    Some(s) => s,
                    None => &ResolvedType::Base(name),
                };
                if let Some(t) = self.resolved_types.get(tir_ty) {
                    t
                } else {
                    die!("Unknown type: {ty}");
                }
            }
            RawType::Pointer(p) => {
                let inner = self.check_type(Spanned::new(*p, ty.span));
                self.resolved_types.add(ResolvedType::Pointer(inner.inner))
            }
            RawType::Function { args, returns } => {
                let args = args
                    .into_iter()
                    .map(|a| {
                        let arg_ty = self.check_type(Spanned::new(*a, ty.span));
                        if *arg_ty.inner.lookup() == ResolvedType::Void {
                            die!("Argument cannot have type `void`: {arg_ty}")
                        }
                        println!("{arg_ty}");
                        arg_ty.inner
                    })
                    .collect();
                let returns = self.check_type(Spanned::new(*returns, ty.span)).inner;
                self.resolved_types
                    .add(ResolvedType::Function { args, returns })
            }
        };
        Spanned::new(id, ty.span)
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
                self.env.push_scope();
                let mut checked_args = vec![];
                let mut arg_types = vec![];
                for (i, (argname, ty)) in args.into_iter().enumerate() {
                    let var_ty = self.check_type(ty);
                    if *var_ty.inner.lookup() == ResolvedType::Void {
                        die!("Function argument cannot have type {var_ty}");
                    }

                    let argsym = self.add_local_symbol(argname, var_ty.inner, SymbolKind::Param(i));
                    checked_args.push((argsym, var_ty));
                    arg_types.push(var_ty.inner);
                }
                let returns = self.check_type(returns);

                let ty = ResolvedType::Function {
                    args: arg_types,
                    returns: returns.inner,
                };
                let id = self.resolved_types.add(ty);

                let function_symbol = self.add_global_symbol(name, id, SymbolKind::Function);

                let body = Box::new(self.check_stmt(*body));
                self.env.pop_scope();
                TirObj::Fn {
                    name: function_symbol,
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
                let var_id = lhs.map(|name| self.next_sym(*name));
                let var_ty = rhs.inner.ty;
                // Insert it into this function's context:
                // add to env & mark it as a local variable
                self.env.insert(lhs.inner, var_id.inner);
                self.global_symbols.insert(
                    var_id.inner,
                    SymbolInfo {
                        raw_name: lhs,
                        ty: var_ty,
                        kind: SymbolKind::Local,
                        address_taken: false, // A variable starts non-addr-taken
                    },
                );

                TirStmt::Let {
                    lhs: var_id,
                    ty,
                    rhs,
                }
            }
            HirStmt::While { cond, body } => todo!(),
            HirStmt::Continue => todo!(),
            HirStmt::Break => todo!(),
            HirStmt::If { cond, then_, else_ } => {
                let cond = self.check_expr(cond, None);
                let cond_ty = cond.inner.ty.lookup();
                if *cond_ty != ResolvedType::Bool {
                    die!(
                        "Type mismatch. Expected `{}` but got `{}`: {}",
                        ResolvedType::Bool,
                        cond_ty,
                        cond.span
                    )
                }
                let then_ = Box::new(self.check_stmt(*then_));
                let else_ = Box::new(self.check_stmt(*else_));
                TirStmt::If { cond, then_, else_ }
            }
            HirStmt::Return(val) => {
                let ResolvedType::Function { returns, .. } =
                    *self.get_symbol_info(self.func.raw_name).ty.lookup()
                else {
                    panic!("Function with non-function type");
                };
                let checked_val = self.check_expr(val, Some(returns));
                if checked_val.inner.ty != returns {
                    die!(
                        "Mismatched return type. Function {} expects {returns} but got {}: {}",
                        self.func.raw_name.inner,
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
        // All statements have type void
        let ty = self.resolved_types.add(ResolvedType::Void);
        // Inherit the same span from the previous phase
        Spanned::new(kind, span)
    }

    fn check_expr(
        &mut self,
        Spanned { inner: expr, span }: Spanned<HirExpr>,
        hint: Option<ResolvedTypeId>,
    ) -> Spanned<TirExpr> {
        let inner = match expr {
            HirExpr::Void => {
                let kind = TirExprKind::Void;
                TirExpr::new(kind, self.resolved_types.add(ResolvedType::Void))
            }
            HirExpr::Num(int_str) => {
                let ty = match hint {
                    Some(hint_id) => {
                        let hint_ty = hint_id.lookup();
                        if hint_ty.is_integral() {
                            hint_id
                        } else {
                            self.resolved_types.add(ResolvedType::I32)
                        }
                    }
                    None => self.resolved_types.add(ResolvedType::I32),
                };
                let result = match ty.lookup() {
                    ResolvedType::I8 => int_str.parse::<i8>().map(|i| i as i128),
                    ResolvedType::U8 => int_str.parse::<u8>().map(|i| i as i128),
                    ResolvedType::I16 => int_str.parse::<i16>().map(|i| i as i128),
                    ResolvedType::U16 => int_str.parse::<u16>().map(|i| i as i128),
                    ResolvedType::I32 => int_str.parse::<i32>().map(|i| i as i128),
                    ResolvedType::U32 => int_str.parse::<u32>().map(|i| i as i128),
                    ResolvedType::I64 => int_str.parse::<i64>().map(|i| i as i128),
                    ResolvedType::U64 => int_str.parse::<u64>().map(|i| i as i128),
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
                let ty = self.resolved_types.add(ResolvedType::Bool);
                let kind = TirExprKind::Bool(b);
                TirExpr::new(kind, ty)
            }
            HirExpr::Ident(i) => {
                let Some(symbol) = self.env.get(&i) else {
                    die!("Undefined variable: {i}");
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
                // if !checked_lhs.inner.kind.is_valid_lvalue() {
                //     die!(
                //         "Cannot assign to this expression as it is not a valid LVALUE: {}, {:?}",
                //         checked_lhs.span,
                //         checked_lhs.inner.kind
                //     )
                // }
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
                let u64_ty = self.resolved_types.add(ResolvedType::U64);
                let index = self.check_expr(*index, Some(u64_ty));
                let index_ty = index.inner.ty;
                if *index_ty.lookup() != ResolvedType::U64 {
                    die!("Cannot index a pointer with a {index_ty}. Use a u64 instead: {index}")
                }
                let ResolvedType::Pointer(elem_ty) = *expr.inner.ty.lookup() else {
                    die!("Cannot index a non-pointer type {}: {expr}", expr.inner.ty)
                };
                let span = expr.span;
                let sizeof = Spanned::new(
                    TirExpr::new(TirExprKind::Num(elem_ty.lookup().size() as i128), u64_ty),
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
                    ResolvedType::Pointer(id) => Some(*id),
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
                            ResolvedType::Pointer(id) => *id,
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
                        let TirExprKind::Ident(varname) = checked_rhs.inner.kind else {
                            unreachable!("rhs was shown to be an Ident")
                        };

                        // Given &x, where x: T, &x has type *T
                        let ty = self.resolved_types.add(ResolvedType::Pointer(rhs_ty));

                        // Mark this symbol as address-taken
                        let Some(info) = self.global_symbols.get_mut(&varname) else {
                            die!("Undefined variable {varname}");
                        };
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
                        if *(rhs_ty.lookup()) != ResolvedType::Bool {
                            die!("Cannot logical not a `{rhs_ty}`: {span}")
                        }
                        rhs_ty
                    }
                    UnOp::Neg => {
                        if !(rhs_ty.lookup()).is_signed() {
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
                        if (lhs_ty != rhs_ty) || !(lhs_ty.lookup()).is_integral() {
                            die!("Cannot add `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Sub => {
                        if (lhs_ty != rhs_ty) || !(lhs_ty.lookup()).is_integral() {
                            die!("Cannot subtract `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Mul => {
                        if (lhs_ty != rhs_ty) || !(lhs_ty.lookup()).is_integral() {
                            die!("Cannot multiply `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Div => {
                        if (lhs_ty != rhs_ty) || !(lhs_ty.lookup()).is_integral() {
                            die!("Cannot divide `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Eq => {
                        if lhs_ty != rhs_ty {
                            die!("Cannot compare `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        self.resolved_types.add(ResolvedType::Bool)
                    }
                    BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                        if (lhs_ty != rhs_ty) || !(lhs_ty.lookup()).is_integral() {
                            die!("Cannot compare `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        self.resolved_types.add(ResolvedType::Bool)
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
                // TODO: once we add all functions to compiler context, we can check if the user is
                // passing in the right types
                let callee = Box::new(self.check_expr(*callee, hint));
                let args = args.into_iter().map(|a| self.check_expr(a, None)).collect();
                let kind = TirExprKind::Call { callee, args };
                TirExpr::new(kind, todo!())
            }
            HirExpr::SizeOfTy { ty } => {
                let ty_size = self.check_type(ty).inner.lookup().size();
                let kind = TirExprKind::Num(ty_size as i128);
                let ty = self.resolved_types.add(ResolvedType::U64);
                TirExpr::new(kind, ty)
            }
            HirExpr::SizeOfExpr { expr } => {
                let ty_size = self.check_expr(*expr, None).inner.ty.lookup().size();
                let kind = TirExprKind::Num(ty_size as i128);
                let ty = self.resolved_types.add(ResolvedType::U64);
                TirExpr::new(kind, ty)
            }
        };
        Spanned::new(inner, span)
    }
}
