use crate::{
    ast::*,
    compiler::{Compiler, SymbolInfo, SymbolKind},
    lir::*,
    tir::*,
};

use Instr::*;

impl Compiler {
    #[inline(always)]
    fn next_reg(&mut self) -> VReg {
        let id = self.curr_fn.reg_count;
        self.curr_fn.reg_count += 1;
        VReg(id)
    }

    fn next_bb(&mut self, name: Option<&'static str>) -> BB {
        let id = self.bb_count;
        self.bb_count += 1;
        BB(name.unwrap_or(".L"), id)
    }

    fn start_new_block(&mut self, name: BB) {
        if let Some(old_name) = self.curr_fn.curr_bb_name {
            // Commit the old block, but first check if it terminated
            let buf = std::mem::take(&mut self.curr_fn.buf);
            let terminator = match self.get_terminator() {
                Some(t) => t,
                None => Jmp(name), // if it didn't terminate, hook it up to the new one
            };
            let bb = BasicBlock::new(old_name, buf, terminator);
            self.curr_fn.bbs.push(bb);
        }
        self.curr_fn.curr_bb_name = Some(name);
    }

    fn get_terminator(&self) -> Option<Instr> {
        match self.curr_fn.buf.last() {
            Some(i) => match i {
                RetVoid | Ret(..) | Br(..) | Jmp(..) => Some(*i),
                _ => None,
            },
            None => None,
        }
    }

    fn emit(&mut self, instr: Instr) {
        if self.get_terminator().is_none() {
            self.curr_fn.buf.push(instr);
        }
    }

    pub fn lower_func(&mut self, obj: TirObj) -> LIRFunction {
        match obj.kind {
            TirObjKind::Fn {
                name,
                returns,
                args,
                body,
            } => {
                let entry_bb = self.next_bb(Some("entry"));
                self.start_new_block(entry_bb);
                let symbol_table = std::mem::take(&mut self.curr_fn.symbol_table);
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
                            let dst = self.next_reg();
                            self.emit(Alloc(ty, dst, name));
                            self.curr_fn.var2val.insert(name, VVal::Ptr(dst));
                        }
                        SymbolKind::Param(num) => {
                            let dst = self.next_reg();
                            self.emit(Param(ty, dst, num, name));
                            if address_taken {
                                let new_dst = self.next_reg();
                                self.emit(Alloc(ty, new_dst, name));
                                self.emit(Store(ty, new_dst, dst));
                                self.curr_fn.var2val.insert(name, VVal::Ptr(new_dst));
                            } else {
                                self.curr_fn.var2val.insert(name, VVal::Reg(dst));
                            }
                        }
                        SymbolKind::Global => todo!(),
                        SymbolKind::Function => todo!(),
                    }
                }

                let body_bb = self.next_bb(Some("body"));
                self.start_new_block(body_bb);
                self.lower_stmt(*body);

                if self.get_terminator().is_none() && !self.curr_fn.buf.is_empty() {
                    match returns.inner.lookup() {
                        TirType::Void => self.curr_fn.buf.push(RetVoid),
                        _ => panic!(
                            "Function was expected to return {returns}but control flow reaches the end of the function"
                        ),
                    }
                }

                let exit_bb = self.next_bb(None);
                self.start_new_block(exit_bb);

                LIRFunction::new(
                    name.inner,
                    std::mem::take(&mut self.curr_fn.bbs),
                    self.curr_fn.reg_count,
                )
            }
            _ => unreachable!(),
        }
    }

    fn lower_stmt(&mut self, stmt: Spanned<TirStmt>) {
        match stmt.inner.kind {
            TirStmtKind::Let { lhs, ty, rhs } => {
                let VVal::Ptr(rs1) = self.curr_fn.var2val.get(&lhs.inner).copied().unwrap() else {
                    panic!("Local variable must be an alloca'd pointer");
                };
                let ty = rhs.inner.meta.into();
                let rs2 = self.lower_expr(rhs);
                self.emit(Store(ty, rs1, rs2));
            }
            TirStmtKind::While { cond, body } => todo!(),
            TirStmtKind::Continue => todo!(),
            TirStmtKind::Break => todo!(),
            TirStmtKind::If { cond, then_, else_ } => {
                // PREVIOUS BB
                let if_bb = self.next_bb(Some("if"));
                let then_bb = self.next_bb(Some("then"));
                let else_bb = self.next_bb(Some("else"));
                let end_bb = self.next_bb(Some("endif"));

                // IF BB
                self.start_new_block(if_bb);
                let cond_val = self.lower_expr(cond);
                let jmp_else = Br(cond_val, then_bb, else_bb);
                self.emit(jmp_else);

                // THEN BB
                self.start_new_block(then_bb);
                self.lower_stmt(*then_);
                let jmp_end = Jmp(end_bb);
                self.emit(jmp_end);

                // ELSE BB
                self.start_new_block(else_bb);
                self.lower_stmt(*else_);
                let jmp_end = Jmp(end_bb);
                self.emit(jmp_end);

                // END BB (empty)
                self.start_new_block(end_bb);
            }
            TirStmtKind::Return(spanned) => {
                let ty = spanned.inner.meta.into();
                let rs1 = self.lower_expr(spanned);
                self.emit(Ret(ty, rs1));
            }
            TirStmtKind::Block(spanneds) => {
                for s in spanneds {
                    self.lower_stmt(s);
                }
            }
            TirStmtKind::Expr(e) => {
                self.lower_expr(e);
            }
        }
    }

    fn lower_expr(&mut self, expr: Spanned<TirExpr>) -> VReg {
        let dst = self.next_reg();
        match expr.inner.kind {
            TirExprKind::Void => dst,
            TirExprKind::Num(imm) => {
                let ty = expr.inner.meta.into();
                self.emit(Const(ty, dst, imm));
                dst
            }
            TirExprKind::Bool(b) => {
                let ty = expr.inner.meta.into();
                self.emit(Const(ty, dst, b as i128));
                dst
            }
            TirExprKind::Ident(varname) => {
                let ty = expr.inner.meta.into();
                let Some(val) = self.curr_fn.var2val.get(&varname).copied() else {
                    panic!("Undefined variable: {varname}");
                };
                match val {
                    VVal::Ptr(rs1) => {
                        self.emit(Load(ty, dst, rs1));
                        dst
                    }
                    VVal::Reg(rs1) => rs1,
                }
            }
            TirExprKind::Un { op, rhs } => {
                let ty = expr.inner.meta.into();
                let rs1 = self.lower_expr(*rhs);
                match op {
                    UnOp::Not => todo!(),
                    UnOp::Neg => {
                        let rs2 = self.next_reg();
                        self.emit(Const(ty, rs2, -1));
                        self.emit(Muls(ty, dst, rs1, rs2));
                    }
                }
                dst
            }
            TirExprKind::Bin { op, lhs, rhs } => {
                let ty = expr.inner.meta.into();
                let is_signed = lhs.inner.meta.lookup().is_signed();
                let rs1 = self.lower_expr(*lhs);
                let rs2 = self.lower_expr(*rhs);
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
                self.emit(instr);
                dst
            }
            TirExprKind::Assign { lhs, rhs } => match lhs.inner.kind {
                ExprKind::Ident(varname) => {
                    let Some(val) = self.curr_fn.var2val.get(&varname).copied() else {
                        panic!("Lvar not found: {varname}");
                    };
                    let ty = rhs.inner.meta.into();
                    let rs2 = self.lower_expr(*rhs);
                    match val {
                        VVal::Ptr(rs1) => {
                            self.emit(Store(ty, rs1, rs2));
                        }
                        VVal::Reg(rs1) => {
                            self.emit(Copy(ty, rs1, rs2));
                        }
                    }
                    rs2
                }
                ExprKind::Deref { rhs: store_target } => {
                    let ty = rhs.inner.meta.into();
                    let rs1 = self.lower_expr(*store_target);
                    let rs2 = self.lower_expr(*rhs);
                    self.emit(Store(ty, rs1, rs2));
                    rs1
                }
                _ => unreachable!(),
            },
            TirExprKind::Deref { rhs } => {
                let ty = expr.inner.meta.into();
                let rs1 = self.lower_expr(*rhs);
                self.emit(Load(ty, dst, rs1));
                dst
            }
            // AddrOf is a meta-instruction. It doesn't actually produce any "work" per-se.
            // It simply grabs an existing pointer to the named storage and returns that for use by
            // other expressions
            TirExprKind::AddrOf { rhs } => {
                let ExprKind::Ident(varname) = rhs.inner.kind else {
                    unreachable!()
                };
                let Some(val) = self.curr_fn.var2val.get(&varname).copied() else {
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
