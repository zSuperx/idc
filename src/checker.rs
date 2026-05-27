use crate::hir::*;
use crate::tir::*;
use crate::{ast::*, compiler::Compiler, hir, tir::Id};

impl Compiler {
    fn check_type(&mut self, ty: Spanned<hir::HirType>) -> Spanned<Id> {
        let id = match ty.inner {
            hir::HirType::Named(name) => match name {
                "i32" => get_type(&TirType::I32).unwrap(),
                "u32" => get_type(&TirType::U32).unwrap(),
                "bool" => get_type(&TirType::Bool).unwrap(),
                "void" => get_type(&TirType::Void).unwrap(),
                foreign => {
                    if let Some(t) = get_type(&TirType::Base(foreign)) {
                        t
                    } else {
                        panic!("Unknown type: `{ty}`");
                    }
                }
            },
            hir::HirType::Pointer(p) => {
                let inner = self.check_type(Spanned::new(*p, ty.span));
                TirType::Pointer(inner.inner).id()
            }
            hir::HirType::Function { args, returns } => {
                let args = args
                    .into_iter()
                    .map(|a| self.check_type(Spanned::new(a, ty.span)).inner)
                    .collect();
                let returns = self.check_type(Spanned::new(*returns, ty.span)).inner;
                TirType::Function { args, returns }.id()
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
                self.type_env.push_scope();
                let mut checked_args = vec![];
                let mut arg_types = vec![];
                for (argname, ty) in args {
                    let ty = self.check_type(ty);
                    let None = self.type_env.insert(argname.inner, ty.inner) else {
                        panic!("Duplicate parameter: {argname}");
                    };
                    checked_args.push((argname, ty));
                    arg_types.push(ty.inner);
                }
                let body = Box::new(self.check_stmt(*body));
                let returns = self.check_type(returns);
                self.type_env.pop_scope();
                let ty = TirType::Function {
                    args: arg_types,
                    returns: returns.inner,
                };
                let id = ty.id();
                let kind = TirObjKind::Fn {
                    name,
                    returns,
                    body,
                    args: checked_args,
                    lvars: std::mem::take(&mut self.current_fn_lvars),
                };
                TirObj::new(kind, id)
            }
            HirObjKind::Global { lhs, rhs } => todo!(),
            HirObjKind::Struct { name, fields } => todo!(),
        };
        Spanned::new(inner, span)
    }

    fn check_stmt(&mut self, Spanned { inner, span }: Spanned<HirStmt>) -> Spanned<TirStmt> {
        let inner = match inner.kind {
            HirStmtKind::Let { lhs, ty, rhs } => {
                let ty = ty.map(|t| self.check_type(t));
                let rhs = self.check_expr(rhs, ty);
                if let Some(ty) = ty
                    && ty.inner != rhs.inner.meta
                {
                    panic!(
                        "Type mismatch. Expected `{}` \n...but got `{}`: {}",
                        ty, rhs.inner.meta, rhs.span
                    );
                }
                self.type_env.insert(lhs.inner, rhs.inner.meta);
                self.current_fn_lvars.push((lhs.inner, rhs.inner.meta));
                let kind = TirStmtKind::Let { lhs, ty, rhs };
                TirStmt::new(kind, TirType::Void.id())
            }
            HirStmtKind::While { cond, body } => todo!(),
            HirStmtKind::Continue => todo!(),
            HirStmtKind::Break => todo!(),
            HirStmtKind::If { cond, then_, else_ } => {
                let cond = self.check_expr(cond, None);
                let cond_ty = lookup_type(cond.inner.meta);
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
                let kind = TirStmtKind::If { cond, then_, else_ };
                TirStmt::new(kind, TirType::Void.id())
            }
            HirStmtKind::Return(val) => {
                todo!()
            }
            HirStmtKind::Block(s) => {
                self.type_env.push_scope();
                let kind =
                    TirStmtKind::Block(s.into_iter().map(|st| self.check_stmt(st)).collect());
                self.type_env.pop_scope();
                TirStmt::new(kind, TirType::Void.id())
            }
            HirStmtKind::Expr(e) => {
                let e = self.check_expr(e, None);
                let kind = TirStmtKind::Expr(e);
                TirStmt::new(kind, TirType::Void.id())
            }
        };
        Spanned::new(inner, span)
    }

    fn check_expr(
        &mut self,
        Spanned { inner, span }: Spanned<HirExpr>,
        hint: Option<Spanned<Id>>,
    ) -> Spanned<TirExpr> {
        let inner = match inner.kind {
            HirExprKind::Num(int_str) => {
                let ty = match hint {
                    Some(hint_id) => {
                        let hint_ty = lookup_type(hint_id.inner);
                        if hint_ty.is_integral() {
                            hint_id.inner
                        } else {
                            TirType::I32.id()
                        }
                    }
                    None => TirType::I32.id(),
                };
                let result = match lookup_type(ty) {
                    TirType::I32 => int_str.parse::<i32>().map(|i| i as i128),
                    TirType::U32 => int_str.parse::<u32>().map(|i| i as i128),
                    _ => {
                        panic!("`{int_str}` could not be parsed as a {ty}");
                    }
                };
                let Ok(num) = result else {
                    panic!("`{int_str}` could not be parsed as a {ty}");
                };
                let kind = TirExprKind::Num(num);
                TirExpr::new(kind, ty)
            }
            HirExprKind::Bool(b) => {
                let ty = TirType::Bool.id();
                let kind = TirExprKind::Bool(b);
                TirExpr::new(kind, ty)
            }
            HirExprKind::Ident(i) => {
                let Some(ty) = self.type_env.get(i) else {
                    panic!("Variable `{i}` used but not defined: {span}");
                };
                let kind = TirExprKind::Ident(i);
                TirExpr::new(kind, ty)
            }
            HirExprKind::Assign { lhs, rhs } => {
                let checked_lhs = self.check_expr(*lhs, hint);
                let lhs_ty = checked_lhs.inner.meta;
                let checked_rhs = self.check_expr(*rhs, hint);
                let rhs_ty = checked_rhs.inner.meta;
                if lhs_ty != rhs_ty {
                    panic!("Cannot assign a `{rhs_ty}` to `{lhs_ty}`: {span}")
                }
                if !checked_rhs.inner.kind.is_valid_lvalue() {
                    panic!(
                        "Cannot assign to this expression as it is not a valid LVALUE: {}",
                        checked_lhs.span
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
                todo!()
            }
            HirExprKind::AddrOf { rhs } => {
                todo!()
            }
            HirExprKind::Un { op, rhs } => {
                let checked_rhs = self.check_expr(*rhs, hint);
                let rhs_ty = checked_rhs.inner.meta;
                let ty = match op {
                    UnOp::Not => {
                        if *lookup_type(rhs_ty) != TirType::Bool {
                            panic!("Cannot logical not a `{rhs_ty}`: {span}")
                        }
                        rhs_ty
                    }
                    UnOp::Neg => {
                        if !lookup_type(rhs_ty).is_signed() {
                            panic!("Cannot negate not a `{rhs_ty}`: {span}")
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
                let checked_rhs = self.check_expr(*rhs, hint);
                let rhs_ty = checked_rhs.inner.meta;
                let ty = match op {
                    BinOp::Add => {
                        if (lhs_ty != rhs_ty) || !lookup_type(lhs_ty).is_integral() {
                            panic!("Cannot add `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Sub => {
                        if (lhs_ty != rhs_ty) || !lookup_type(lhs_ty).is_integral() {
                            panic!("Cannot subtract `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Mul => {
                        if (lhs_ty != rhs_ty) || !lookup_type(lhs_ty).is_integral() {
                            panic!("Cannot multiply `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Div => {
                        if (lhs_ty != rhs_ty) || !lookup_type(lhs_ty).is_integral() {
                            panic!("Cannot divide `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        lhs_ty
                    }
                    BinOp::Eq => {
                        if lhs_ty != rhs_ty {
                            panic!("Cannot compare `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        TirType::Bool.id()
                    }
                    BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                        if (lhs_ty != rhs_ty) || !lookup_type(lhs_ty).is_integral() {
                            panic!("Cannot compare `{lhs_ty}` and `{rhs_ty}`: {span}")
                        }
                        TirType::Bool.id()
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
