use crate::{
    CFG,
    arch::lir::*,
    ast::*,
    aux::{Compiler, SymbolInfo, SymbolKind},
    tir::*,
};

use crate::prelude::*;

use LirInstr::*;

impl Compiler {
    pub fn lower_func(
        &mut self,
        Spanned { inner: obj, span }: Spanned<TirObj>,
    ) -> Builder<LirInstr> {
        match obj {
            TirObj::Fn {
                name,
                returns,
                args,
                body,
            } => {
                let mut builder = Builder::new(self.func.raw_name.inner, 0, 0);
                let entry_bb = builder.new_bb("entry");
                builder.start_new_block(entry_bb);
                let symbol_table = std::mem::take(&mut self.global_symbols);
                for (name, info) in symbol_table.iter() {
                    let name = *name;
                    let SymbolInfo {
                        raw_name,
                        ty,
                        kind,
                        address_taken,
                    } = *info;
                    match kind {
                        SymbolKind::Local => {
                            let dst = LirVal::mem(builder.new_reg(), ty.size());
                            builder.emit(Alloc(ty.into(), dst, name));
                            self.func.var2val.insert(name, dst);
                        }
                        SymbolKind::Param(num) => {
                            if address_taken {
                                let dst = LirVal::mem(builder.new_reg(), ty.size());
                                builder.emit(Sparam(ty.into(), dst, name, num));
                                self.func.var2val.insert(name, dst);
                            } else {
                                let dst = LirVal::reg(builder.new_reg(), ty.size());
                                builder.emit(Param(ty.into(), dst, name, num));
                                self.func.var2val.insert(name, dst);
                            }
                        }
                        SymbolKind::Global => todo!(),
                        SymbolKind::Function => todo!(),
                    }
                }

                let body_bb = builder.new_bb("body");
                builder.start_new_block(body_bb);
                self.lower_stmt(&mut builder, *body);

                if builder.get_terminator().is_none() {
                    match returns.inner.lookup() {
                        ResolvedType::Void => builder.buf.push(Retv),
                        _ => die!(
                            "Control flow reaches the end of a function that was expected to return {returns}"
                        ),
                    }
                }

                // This is done just to flush any instructions in the Basic Block buffer
                let exit_bb = builder.new_bb("");
                builder.start_new_block(exit_bb);

                // Do a final loop to compute each blocks' successors from its terminator
                // instruction
                for bb in builder.bbs.iter_mut() {
                    match &bb.terminator {
                        Br(ty, lir_val, tgt1, tgt2) => bb.succ.extend_from_slice(&[*tgt1, *tgt2]),
                        Jmp(tgt) => bb.succ.push(*tgt),
                        Retv => {}
                        Ret(..) => {}
                        x => die!("How did this end up as a terminator? {x}"),
                    }
                }

                builder
            }
            _ => unreachable!(),
        }
    }

    fn lower_stmt(
        &mut self,
        builder: &mut Builder<LirInstr>,
        Spanned { inner: stmt, span }: Spanned<TirStmt>,
    ) {
        // TODO: Change this its kind of stupid. Span::content() should return just the source code
        // slice, whereas Span::to_string() should print the file:row:col, the content, and the
        // arrows
        if CFG.verbose {
            builder.emit(Comment(
                span.content().split('\n').nth(3).unwrap().to_string(),
            ));
        }
        match stmt {
            TirStmt::Let { ty, lhs, rhs } => {
                let rs1 = self.func.var2val.get(&lhs.inner).copied().unwrap();
                let LirValKind::Mem(..) = rs1.kind else {
                    die!("Local variable must be an alloca'd pointer");
                };
                let ty = rhs.inner.ty;
                let rs2 = self.lower_expr(builder, rhs);
                builder.emit(Store(ty, rs1, rs2));
            }
            TirStmt::While { cond, body } => todo!(),
            TirStmt::Continue => todo!(),
            TirStmt::Break => todo!(),
            TirStmt::If { cond, then_, else_ } => {
                // Create labels
                let if_bb = builder.new_bb("if");
                let then_bb = builder.new_bb("then");
                let else_bb = builder.new_bb("else");
                let end_bb = builder.new_bb("endif");

                // IF BB
                builder.start_new_block(if_bb);
                let cond_val = self.lower_expr(builder, cond);
                let branch = Br(
                    self.resolved_types.add(ResolvedType::I8),
                    cond_val,
                    then_bb,
                    else_bb,
                );
                builder.emit(branch);

                // THEN BB
                builder.start_new_block(then_bb);
                self.lower_stmt(builder, *then_);
                let jmp_end = Jmp(end_bb);
                builder.emit(jmp_end);
                let then_term = builder.get_terminator();

                // ELSE BB
                builder.start_new_block(else_bb);
                self.lower_stmt(builder, *else_);
                let jmp_end = Jmp(end_bb);
                builder.emit(jmp_end);
                let else_term = builder.get_terminator();

                // END BB (empty)
                match (then_term, else_term) {
                    (Some(t), Some(e)) if is_ret(&t) && is_ret(&e) => {}
                    _ => builder.start_new_block(end_bb),
                }
            }
            TirStmt::Return(spanned) => {
                if *spanned.inner.ty == ResolvedType::Void {
                    builder.emit(Retv);
                } else {
                    let ty = spanned.inner.ty;
                    let rs1 = self.lower_expr(builder, spanned);
                    builder.emit(Ret(ty, rs1));
                }
            }
            TirStmt::Block(spanneds) => {
                for s in spanneds {
                    self.lower_stmt(builder, s);
                }
            }
            TirStmt::Expr(e) => {
                self.lower_expr(builder, e);
            }
        }
    }

    fn lower_expr(
        &mut self,
        builder: &mut Builder<LirInstr>,
        Spanned { inner: expr, span }: Spanned<TirExpr>,
    ) -> LirVal {
        let ty = expr.ty;
        let size = ty.size();
        let dst = LirVal::reg(builder.new_reg(), size);
        match expr.kind {
            TirExprKind::Void => dst,
            TirExprKind::Num(imm) => LirVal::imm(imm, size),
            TirExprKind::Bool(b) => {
                let imm = b as i128;
                LirVal::imm(imm, size)
            }
            TirExprKind::Ident(varname) => {
                let Some(rs1) = self.func.var2val.get(&varname).copied() else {
                    die!("Undefined variable: {varname}");
                };
                match rs1.kind {
                    LirValKind::Mem(..) => {
                        builder.emit(Load(ty, dst, rs1));
                        dst
                    }
                    _ => rs1,
                }
            }
            TirExprKind::Un { op, rhs } => {
                let rs1 = self.lower_expr(builder, *rhs);
                match op {
                    UnOp::Not => todo!(),
                    UnOp::Neg => {
                        builder.emit(Smul(ty, dst, rs1, LirVal::imm(-1, size)));
                    }
                }
                dst
            }
            TirExprKind::Bin { op, lhs, rhs } => {
                let is_signed = lhs.inner.ty.is_signed();
                let rs1 = self.lower_expr(builder, *lhs);
                let rs2 = self.lower_expr(builder, *rhs);
                let instr = match (op, is_signed) {
                    (BinOp::Add, _) => Add(ty, dst, rs1, rs2),
                    (BinOp::Sub, _) => Sub(ty, dst, rs1, rs2),
                    (BinOp::Mul, true) => Smul(ty, dst, rs1, rs2),
                    (BinOp::Mul, false) => Umul(ty, dst, rs1, rs2),
                    (BinOp::Div, true) => todo!(),
                    (BinOp::Div, false) => todo!(),
                    (BinOp::Eq, _) => Eq(ty, dst, rs1, rs2),
                    (BinOp::Le, true) => Sle(ty, dst, rs1, rs2),
                    (BinOp::Le, false) => Ule(ty, dst, rs1, rs2),
                    (BinOp::Lt, true) => Slt(ty, dst, rs1, rs2),
                    (BinOp::Lt, false) => Ult(ty, dst, rs1, rs2),
                    (BinOp::Ge, true) => Sge(ty, dst, rs1, rs2),
                    (BinOp::Ge, false) => Uge(ty, dst, rs1, rs2),
                    (BinOp::Gt, true) => Sgt(ty, dst, rs1, rs2),
                    (BinOp::Gt, false) => Ugt(ty, dst, rs1, rs2),
                };
                builder.emit(instr);
                dst
            }
            TirExprKind::Assign { lhs, rhs } => match lhs.inner.kind {
                TirExprKind::Ident(varname) => {
                    let Some(rs1) = self.func.var2val.get(&varname).copied() else {
                        die!("Lvar not found: {varname}");
                    };
                    let rs2 = self.lower_expr(builder, *rhs);
                    match rs1.kind {
                        LirValKind::Mem(..) => {
                            builder.emit(Store(ty, rs1, rs2));
                        }
                        _ => {
                            builder.emit(Copy(ty, rs1, rs2));
                        }
                    }
                    rs2
                }
                TirExprKind::Deref { rhs: store_target } => {
                    let rs1 = self.lower_expr(builder, *store_target);
                    let rs2 = self.lower_expr(builder, *rhs);
                    builder.emit(Store(ty, rs1, rs2));
                    rs1
                }
                _ => unreachable!(),
            },
            TirExprKind::Deref { rhs } => {
                let rs1 = self.lower_expr(builder, *rhs);
                builder.emit(Load(ty, dst, rs1));
                dst
            }
            // AddrOf is a ty-instruction. It doesn't actually produce any "work" per-se.
            // It simply grabs an existing pointer to the named storage and returns that for use by
            // other expressions
            TirExprKind::AddrOf { rhs } => {
                let TirExprKind::Ident(varname) = rhs.inner.kind else {
                    unreachable!()
                };
                let Some(rs1) = self.func.var2val.get(&varname).copied() else {
                    die!("Lvar not found: {varname}");
                };
                let LirValKind::Mem(..) = rs1.kind else {
                    panic!("All named storage expressions should be allocated on the stack by now")
                };
                rs1
            }
            TirExprKind::Call { callee, args } => todo!(),
            TirExprKind::Cast { target_ty, rhs } => {
                let ty = rhs.inner.ty;
                let rhs = self.lower_expr(builder, *rhs);
                if ty.size() == target_ty.inner.size() {
                    // @T(T) is a No-op
                    rhs
                } else {
                    if ty.is_primitive() {
                        if ty.size() < target_ty.inner.size() {
                            if target_ty.inner.is_signed() {
                                builder.emit(Sext(target_ty.inner, dst, rhs));
                            } else {
                                builder.emit(Zext(target_ty.inner, dst, rhs));
                            }
                        } else {
                            builder.emit(Trunc(target_ty.inner, dst, rhs));
                        }
                    } else {
                        todo!()
                    }
                    dst
                }
            }
            TirExprKind::Index { expr, index } => {
                todo!()
            }
            // The size of a type is known at checking time, so the checker literally replaces
            // TirExprKind::SizeOf with Expr::Num, meaning this should never be hit
            TirExprKind::SizeOfTy { .. } | TirExprKind::SizeOfExpr { .. } => unreachable!(),
        }
    }
}

fn is_ret(i: &LirInstr) -> bool {
    matches!(i, LirInstr::Ret(..) | LirInstr::Retv)
}
