use crate::compiler::{SymbolInfo, SymbolKind};
use crate::hir::HirType;
use crate::hir::*;
use crate::tir::*;
use crate::{ast::*, compiler::Compiler, hir, tir::TypeId};

impl Compiler {
    fn next_var(&mut self, argname: &str) -> VarId {
        let id = self.var_count;
        self.var_count += 1;
        self.symbols.add(format!("{argname}_{id}"))
    }

    fn check_type(&mut self, ty: Spanned<HirType>) -> Spanned<TypeId> {
        let id = match ty.inner {
            HirType::Named(name) => {
                let tir_ty = match self.builtin_types.get(name) {
                    Some(s) => s,
                    None => &TirType::Base(name),
                };
                if let Some(t) = self.known_types.get(tir_ty) {
                    t
                } else {
                    panic!("Unknown type: {ty}");
                }
            }
            HirType::Pointer(p) => {
                let inner = self.check_type(Spanned::new(*p, ty.span));
                self.known_types.add(TirType::Pointer(inner.inner))
            }
            HirType::Function { args, returns } => {
                let args = args
                    .into_iter()
                    .map(|a| self.check_type(Spanned::new(a, ty.span)).inner)
                    .collect();
                let returns = self.check_type(Spanned::new(*returns, ty.span)).inner;
                self.known_types.add(TirType::Function { args, returns })
            }
        };
        Spanned::new(id, ty.span)
    }

    pub fn check_obj(&mut self, Spanned { inner, span }: Spanned<HirObj>) -> Spanned<TirObj> {
        let inner = match inner.kind {
            HirObjKind::Fn {
                name,
                returns,
                args,
                body,
                ..
            } => {
                self.curr_fn.env.push_scope();
                let mut checked_args = vec![];
                let mut arg_types = vec![];
                for (i, (argname, ty)) in args.into_iter().enumerate() {
                    let var_id = argname.map(|name| self.next_var(*name));
                    let var_ty = self.check_type(ty);

                    self.curr_fn.symbol_table.insert(
                        var_id.inner,
                        SymbolInfo {
                            name: var_id.inner,
                            ty: var_ty.inner,
                            kind: SymbolKind::Param(i),
                            address_taken: false, // A variable starts non-addr-taken
                        },
                    );

                    checked_args.push((var_id, var_ty));
                    if self
                        .curr_fn
                        .env
                        .insert(argname.inner, (var_id.inner, var_ty.inner))
                        .is_some()
                    {
                        panic!("Duplicate parameter: {argname}");
                    };
                    arg_types.push(var_ty.inner);
                }
                let returns = self.check_type(returns);
                self.curr_fn.return_type = Some(returns);

                let body = Box::new(self.check_stmt(*body));
                self.curr_fn.env.pop_scope();
                let ty = TirType::Function {
                    args: arg_types,
                    returns: returns.inner,
                };
                let id = self.known_types.add(ty);
                let fn_name_id = name.map(|name| self.next_var(*name));
                let kind = TirObjKind::Fn {
                    name: fn_name_id,
                    returns,
                    body,
                    args: checked_args,
                };
                TirObj::new(kind, id)
            }
            HirObjKind::Global { lhs, rhs } => todo!(),
            HirObjKind::Struct { name, fields } => todo!(),
        };
        Spanned::new(inner, span)
    }

    fn check_stmt(&mut self, Spanned { inner, span }: Spanned<HirStmt>) -> Spanned<TirStmt> {
        let kind = match inner.kind {
            HirStmtKind::Let { lhs, ty, rhs } => {
                let ty = ty.map(|t| self.check_type(t));
                let rhs = self.check_expr(rhs, ty.map(|t| t.inner));
                if let Some(ty) = ty
                    && ty.inner != rhs.inner.meta
                {
                    panic!(
                        "Type mismatch. Expected {ty} \n...but got `{}`: {}",
                        rhs.inner.meta, rhs.span,
                    );
                }
                let var_id = lhs.map(|name| self.next_var(*name));
                let var_ty = rhs.inner.meta;
                // Insert it into this function's context:
                // add to env & mark it as a local variable
                self.curr_fn.env.insert(lhs.inner, (var_id.inner, var_ty));
                self.curr_fn.symbol_table.insert(
                    var_id.inner,
                    SymbolInfo {
                        name: var_id.inner,
                        ty: var_ty,
                        kind: SymbolKind::Local,
                        address_taken: false, // A variable starts non-addr-taken
                    },
                );

                TirStmtKind::Let {
                    lhs: var_id,
                    ty,
                    rhs,
                }
            }
            HirStmtKind::While { cond, body } => todo!(),
            HirStmtKind::Continue => todo!(),
            HirStmtKind::Break => todo!(),
            HirStmtKind::If { cond, then_, else_ } => {
                let cond = self.check_expr(cond, None);
                let cond_ty = cond.inner.meta.lookup();
                if *cond_ty != TirType::Bool {
                    panic!(
                        "Type mismatch. Expected `{}` but got `{}`: {}",
                        TirType::Bool,
                        cond_ty,
                        cond.span
                    )
                }
                let then_ = Box::new(self.check_stmt(*then_));
                let else_ = Box::new(self.check_stmt(*else_));
                TirStmtKind::If { cond, then_, else_ }
            }
            HirStmtKind::Return(val) => {
                let fn_ret_type = self.curr_fn.return_type.unwrap();
                let checked_val = self.check_expr(val, Some(fn_ret_type.inner));
                if checked_val.inner.meta != fn_ret_type.inner {
                    panic!(
                        "Mismatched return type. Function {} expects {fn_ret_type}but got {}: {}",
                        self.curr_fn.raw_name.inner, checked_val.inner.meta, checked_val.span
                    )
                }
                TirStmtKind::Return(checked_val)
            }
            HirStmtKind::Block(s) => {
                self.curr_fn.env.push_scope();
                self.curr_fn.env.pop_scope();
                TirStmtKind::Block(s.into_iter().map(|st| self.check_stmt(st)).collect())
            }
            HirStmtKind::Expr(e) => {
                let e = self.check_expr(e, None);
                TirStmtKind::Expr(e)
            }
        };
        // All statements have type void
        let ty = self.known_types.add(TirType::Void);
        let inner = TirStmt::new(kind, ty);
        // Inherit the same span from the previous phase
        Spanned::new(inner, span)
    }

    fn check_expr(
        &mut self,
        Spanned { inner, span }: Spanned<HirExpr>,
        hint: Option<TypeId>,
    ) -> Spanned<TirExpr> {
        let inner = match inner.kind {
            HirExprKind::Void => {
                let kind = TirExprKind::Void;
                TirExpr::new(kind, self.known_types.add(TirType::Void))
            }
            HirExprKind::Num(int_str) => {
                let ty = match hint {
                    Some(hint_id) => {
                        let hint_ty = hint_id.lookup();
                        if hint_ty.is_integral() {
                            hint_id
                        } else {
                            self.known_types.add(TirType::I32)
                        }
                    }
                    None => self.known_types.add(TirType::I32),
                };
                let result = match ty.lookup() {
                    TirType::I8 => int_str.parse::<i8>().map(|i| i as i128),
                    TirType::U8 => int_str.parse::<u8>().map(|i| i as i128),
                    TirType::I16 => int_str.parse::<i16>().map(|i| i as i128),
                    TirType::U16 => int_str.parse::<u16>().map(|i| i as i128),
                    TirType::I32 => int_str.parse::<i32>().map(|i| i as i128),
                    TirType::U32 => int_str.parse::<u32>().map(|i| i as i128),
                    TirType::I64 => int_str.parse::<i64>().map(|i| i as i128),
                    TirType::U64 => int_str.parse::<u64>().map(|i| i as i128),
                    _ => {
                        panic!("`{int_str}` could not be parsed as a {ty}");
                    }
                };
                let Ok(num) = result else {
                    panic!("`{int_str}` could not be parsed as a `{ty}`: {span}");
                };
                let kind = TirExprKind::Num(num);
                TirExpr::new(kind, ty)
            }
            HirExprKind::Bool(b) => {
                let ty = self.known_types.add(TirType::Bool);
                let kind = TirExprKind::Bool(b);
                TirExpr::new(kind, ty)
            }
            HirExprKind::Ident(i) => {
                let Some((var, ty)) = self.curr_fn.env.get(&i) else {
                    panic!("Variable `{i}` used but not defined: {span}");
                };
                let kind = TirExprKind::Ident(var);
                TirExpr::new(kind, ty)
            }
            HirExprKind::Assign { lhs, rhs } => {
                let checked_lhs = self.check_expr(*lhs, hint);
                let mut lhs_ty = checked_lhs.inner.meta;
                let checked_rhs = self.check_expr(*rhs, Some(lhs_ty));
                let rhs_ty = checked_rhs.inner.meta;
                if lhs_ty != rhs_ty {
                    panic!("Cannot assign a `{rhs_ty}` to `{lhs_ty}`: {span}")
                }
                if !checked_lhs.inner.kind.is_valid_lvalue() {
                    panic!(
                        "Cannot assign to this expression as it is not a valid LVALUE: {}, {:?}",
                        checked_lhs.span, checked_lhs.inner.kind
                    )
                }
                let ty = lhs_ty;
                let kind = TirExprKind::Assign {
                    lhs: Box::new(checked_lhs),
                    rhs: Box::new(checked_rhs),
                };
                TirExpr::new(kind, ty)
            }
            HirExprKind::Deref { rhs } => {
                // If we get a hint of *T, the sub-expr should be checked with hint T
                let hint_inner = hint.and_then(|h| match h.lookup() {
                    TirType::Pointer(id) => Some(*id),
                    _ => None,
                });
                // We don't want to check the inner expression yet. First check its kind
                match rhs.inner.kind {
                    // If we're dereferencing an addrof, they cancel out
                    // e.g. (*&y == y). In this case, pull out the AddrOf's sub-expr
                    // and check it separately. We do this to avoid the AddrOf book-keeping code
                    ExprKind::AddrOf { rhs } => self.check_expr(*rhs, hint_inner).inner,
                    _ => {
                        let checked_rhs = Box::new(self.check_expr(*rhs, hint_inner));
                        let checked_ty = checked_rhs.inner.meta;
                        // A deref can only happen on a pointer, and its type will be whatever
                        // the pointer is pointing to
                        let ty = match checked_ty.lookup() {
                            TirType::Pointer(id) => *id,
                            _ => panic!(
                                "Cannot dereference non-pointer type `{checked_ty}`: {checked_rhs}"
                            ),
                        };
                        let kind = TirExprKind::Deref { rhs: checked_rhs };
                        TirExpr::new(kind, ty)
                    }
                }
            }
            HirExprKind::AddrOf { rhs } => {
                // We don't want to check the inner expression yet. First check its kind
                match rhs.inner.kind {
                    // AddrOf is kind of a meta-expression. It can ONLY operate on
                    // named values. The exception is AddrOf(Deref(...)), but...
                    ExprKind::Ident(..) => {
                        // If Ident, perform the official check
                        let checked_rhs = Box::new(self.check_expr(*rhs, hint));
                        let rhs_ty = checked_rhs.inner.meta;
                        let ExprKind::Ident(varname) = checked_rhs.inner.kind else {
                            unreachable!("rhs was shown to be an Ident")
                        };

                        // Given &x, where x: T, &x has type *T
                        let ty = self.known_types.add(TirType::Pointer(rhs_ty));

                        // Mark this symbol as address-taken
                        let Some(info) = self.curr_fn.symbol_table.get_mut(&varname) else {
                            panic!("Undefined variable {varname}");
                        };
                        info.address_taken = true;

                        let kind = TirExprKind::AddrOf { rhs: checked_rhs };
                        TirExpr::new(kind, ty)
                    }
                    // ... if we're taking address of a dereference, they cancel out e.g. (&*y == y)
                    ExprKind::Deref { rhs } => self.check_expr(*rhs, hint).inner,
                    _ => panic!("Cannot take the address of this type of expression: {rhs}"),
                }
            }
            HirExprKind::Un { op, rhs } => {
                let checked_rhs = self.check_expr(*rhs, hint);
                let rhs_ty = checked_rhs.inner.meta;
                let ty = match op {
                    UnOp::Not => {
                        if *(rhs_ty.lookup()) != TirType::Bool {
                            panic!("Cannot logical not a `{rhs_ty}`: {span}")
                        }
                        rhs_ty
                    }
                    UnOp::Neg => {
                        if !(rhs_ty.lookup()).is_signed() {
                            panic!("Cannot negate a `{rhs_ty}`: {span}")
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
            HirExprKind::Bin { op, lhs, rhs } => {
                let checked_lhs = self.check_expr(*lhs, hint);
                let lhs_ty = checked_lhs.inner.meta;
                let checked_rhs = self.check_expr(*rhs, Some(lhs_ty));
                let rhs_ty = checked_rhs.inner.meta;
                let ty = match op {
                    BinOp::Add => {
                        if (lhs_ty != rhs_ty) || !(lhs_ty.lookup()).is_integral() {
                            panic!("Cannot add `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Sub => {
                        if (lhs_ty != rhs_ty) || !(lhs_ty.lookup()).is_integral() {
                            panic!("Cannot subtract `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Mul => {
                        if (lhs_ty != rhs_ty) || !(lhs_ty.lookup()).is_integral() {
                            panic!("Cannot multiply `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Div => {
                        if (lhs_ty != rhs_ty) || !(lhs_ty.lookup()).is_integral() {
                            panic!("Cannot divide `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Eq => {
                        if lhs_ty != rhs_ty {
                            panic!("Cannot compare `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        self.known_types.add(TirType::Bool)
                    }
                    BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                        if (lhs_ty != rhs_ty) || !(lhs_ty.lookup()).is_integral() {
                            panic!("Cannot compare `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        self.known_types.add(TirType::Bool)
                    }
                };
                let kind = TirExprKind::Bin {
                    op,
                    lhs: Box::new(checked_lhs),
                    rhs: Box::new(checked_rhs),
                };
                TirExpr::new(kind, ty)
            }
            HirExprKind::Call { callee, args } => todo!(),
        };
        Spanned::new(inner, span)
    }
}
