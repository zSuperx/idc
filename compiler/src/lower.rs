use crate::{
    ast::*,
    aux::{Compiler, SymbolInfo, SymbolKind},
    lir::*,
    tir::*,
};

use LirInstr::*;

impl Compiler {
    pub fn lower_func(&mut self, obj: Spanned<TirObj>) -> Builder<LirInstr> {
        match obj.inner.kind {
            TirObjKind::Fn {
                name,
                returns,
                args,
                body,
            } => {
                let mut builder = Builder::new(self.func.raw_name.inner, 0, 0);
                let entry_bb = builder.next_bb("entry");
                builder.start_new_block(entry_bb);
                let symbol_table = std::mem::take(&mut self.func.symbol_table);
                for (name, info) in symbol_table.iter() {
                    let SymbolInfo {
                        name,
                        ty,
                        kind,
                        address_taken,
                    } = *info;
                    let ty = ty.into();
                    // TODO: alloc params and locals
                    match kind {
                        SymbolKind::Local => {
                            let dst = builder.next_reg();
                            builder.emit(Alloc(ty, dst, name));
                            self.func.var2val.insert(name, VVal::Ptr(dst));
                        }
                        SymbolKind::Param(num) => {
                            let dst = builder.next_reg();
                            builder.emit(Param(ty, dst, num, name));
                            if address_taken {
                                let new_dst = builder.next_reg();
                                builder.emit(Alloc(ty, new_dst, name));
                                builder.emit(Store(ty, new_dst, dst));
                                self.func.var2val.insert(name, VVal::Ptr(new_dst));
                            } else {
                                self.func.var2val.insert(name, VVal::Reg(dst));
                            }
                        }
                        SymbolKind::Global => todo!(),
                        SymbolKind::Function => todo!(),
                    }
                }

                let body_bb = builder.next_bb("body");
                builder.start_new_block(body_bb);
                self.lower_stmt(&mut builder, *body);

                if builder.get_terminator().is_none() {
                    match returns.inner.lookup() {
                        TirType::Void => builder.buf.push(RetVoid),
                        _ => panic!(
                            "Control flow reaches the end of a function that was expected to return {returns}"
                        ),
                    }
                }

                let exit_bb = builder.next_bb("");
                builder.start_new_block(exit_bb);

                builder
            }
            _ => unreachable!(),
        }
    }

    fn lower_stmt(&mut self, builder: &mut Builder<LirInstr>, stmt: Spanned<TirStmt>) {
        match stmt.inner.kind {
            TirStmtKind::Let { lhs, ty, rhs } => {
                let VVal::Ptr(rs1) = self.func.var2val.get(&lhs.inner).copied().unwrap() else {
                    panic!("Local variable must be an alloca'd pointer");
                };
                let ty = rhs.inner.meta.into();
                let rs2 = self.lower_expr(builder, rhs);
                builder.emit(Store(ty, rs1, rs2));
            }
            TirStmtKind::While { cond, body } => todo!(),
            TirStmtKind::Continue => todo!(),
            TirStmtKind::Break => todo!(),
            TirStmtKind::If { cond, then_, else_ } => {
                // PREVIOUS BB
                let if_bb = builder.next_bb("if");
                let then_bb = builder.next_bb("then");
                let else_bb = builder.next_bb("else");
                let end_bb = builder.next_bb("endif");

                // IF BB
                builder.start_new_block(if_bb);
                let cond_val = self.lower_expr(builder, cond);
                let jmp_else = Br(cond_val, then_bb, else_bb);
                builder.emit(jmp_else);

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
                    (Some(t), Some(e)) if t.is_ret() && e.is_ret() => {}
                    _ => builder.start_new_block(end_bb),
                }
            }
            TirStmtKind::Return(spanned) => {
                if *spanned.inner.meta.lookup() == TirType::Void {
                    builder.emit(RetVoid);
                } else {
                    let ty = spanned.inner.meta.into();
                    let rs1 = self.lower_expr(builder, spanned);
                    builder.emit(Ret(ty, rs1));
                }
            }
            TirStmtKind::Block(spanneds) => {
                for s in spanneds {
                    self.lower_stmt(builder, s);
                }
            }
            TirStmtKind::Expr(e) => {
                self.lower_expr(builder, e);
            }
        }
    }

    fn lower_expr(&mut self, builder: &mut Builder<LirInstr>, expr: Spanned<TirExpr>) -> LirVal {
        let dst = builder.next_reg();
        match expr.inner.kind {
            TirExprKind::Void => dst,
            TirExprKind::Num(imm) => LirVal::Imm(imm),
            TirExprKind::Bool(b) => {
                let imm = b as i128;
                LirVal::Imm(imm)
            }
            TirExprKind::Ident(varname) => {
                let ty = expr.inner.meta.into();
                let Some(val) = self.func.var2val.get(&varname).copied() else {
                    panic!("Undefined variable: {varname}");
                };
                match val {
                    VVal::Ptr(rs1) => {
                        builder.emit(Load(ty, dst, rs1));
                        dst
                    }
                    VVal::Reg(rs1) => rs1,
                }
            }
            TirExprKind::Un { op, rhs } => {
                let ty = expr.inner.meta.into();
                let rs1 = self.lower_expr(builder, *rhs);
                match op {
                    UnOp::Not => todo!(),
                    UnOp::Neg => {
                        builder.emit(Muls(ty, dst, rs1, LirVal::Imm(-1)));
                    }
                }
                dst
            }
            TirExprKind::Bin { op, lhs, rhs } => {
                let ty = lhs.inner.meta.into();
                let is_signed = lhs.inner.meta.lookup().is_signed();
                let rs1 = self.lower_expr(builder, *lhs);
                let rs2 = self.lower_expr(builder, *rhs);
                let instr = match (op, is_signed) {
                    (BinOp::Add, _) => Add(ty, dst, rs1, rs2),
                    (BinOp::Sub, _) => Sub(ty, dst, rs1, rs2),
                    (BinOp::Mul, true) => Muls(ty, dst, rs1, rs2),
                    (BinOp::Mul, false) => Mulu(ty, dst, rs1, rs2),
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
                ExprKind::Ident(varname) => {
                    let Some(val) = self.func.var2val.get(&varname).copied() else {
                        panic!("Lvar not found: {varname}");
                    };
                    let ty = rhs.inner.meta.into();
                    let rs2 = self.lower_expr(builder, *rhs);
                    match val {
                        VVal::Ptr(rs1) => {
                            builder.emit(Store(ty, rs1, rs2));
                        }
                        VVal::Reg(rs1) => {
                            builder.emit(Copy(ty, rs1, rs2));
                        }
                    }
                    rs2
                }
                ExprKind::Deref { rhs: store_target } => {
                    let ty = rhs.inner.meta.into();
                    let rs1 = self.lower_expr(builder, *store_target);
                    let rs2 = self.lower_expr(builder, *rhs);
                    builder.emit(Store(ty, rs1, rs2));
                    rs1
                }
                _ => unreachable!(),
            },
            TirExprKind::Deref { rhs } => {
                let ty = expr.inner.meta.into();
                let rs1 = self.lower_expr(builder, *rhs);
                builder.emit(Load(ty, dst, rs1));
                dst
            }
            // AddrOf is a meta-instruction. It doesn't actually produce any "work" per-se.
            // It simply grabs an existing pointer to the named storage and returns that for use by
            // other expressions
            TirExprKind::AddrOf { rhs } => {
                let ExprKind::Ident(varname) = rhs.inner.kind else {
                    unreachable!()
                };
                let Some(val) = self.func.var2val.get(&varname).copied() else {
                    panic!("Lvar not found: {varname}");
                };
                let VVal::Ptr(rs1) = val else {
                    panic!("All named storage expressions should be allocated on the stack by now")
                };
                rs1
            }
            TirExprKind::Call { callee, args } => todo!(),
        }
    }
}
