use crate::ast::{BinOp, Type};
use crate::aux::SymbolKind;
use crate::{CFG, die};
use crate::{IRs::tir::*, aux::Compiler};

use crate::IRs::lir::LirInstr::{self, *};
use crate::common::{IRBuilder, Value};

impl Compiler {
    pub fn lower_func(&mut self, builder: &mut IRBuilder<LirInstr>, obj: &TirObj) {
        match obj {
            TirObj::Fn {
                name,
                returns,
                args,
                body,
            } => {
                let current_function = self.global_symbols.get(name).unwrap();
                let function_name = current_function.raw_name.inner;
                self.current_function = Some(*name);

                builder.create_function(function_name);

                for (sym, info) in self.get_local_symbols_mut() {
                    let val = match info.kind {
                        SymbolKind::Local => {
                            let dst = Value::reg(builder.next_reg());
                            builder.emit(Alloca(info.ty, dst));
                            dst
                        }
                        SymbolKind::Arg(i) => {
                            if info.address_taken {
                                let dst = Value::reg(builder.next_reg());
                                builder.emit(Alloca(info.ty, dst));
                                dst
                            } else {
                                Value::arg(i)
                            }
                        }
                        _ => panic!("What"),
                    };

                    if CFG.verbose {
                        builder.emit(Comment(format!("{val} <- {sym}")));
                    }
                    info.value = Some(val);
                }

                self.lower_stmt(builder, body);
                let default_return_instr = if *returns.lookup() == Type::Void {
                    Some(Retv)
                } else {
                    None
                };
                if !builder.verify(function_name, default_return_instr) {
                    die!(
                        "Control flow reaches end of non-void function: {}",
                        self.lookup_symbol(*name).raw_name
                    );
                }
            }
            TirObj::Global { lhs, rhs } => todo!(),
            TirObj::Struct { name, fields } => todo!(),
        }
    }

    fn lower_stmt(&mut self, builder: &mut IRBuilder<LirInstr>, stmt: &TirStmt) {
        match stmt {
            TirStmt::Let { lhs, ty, rhs } => {
                let rhs_val = self.lower_expr(builder, rhs);
                let info = self.lookup_symbol(*lhs);
                let dst = info.value.expect("Symbol doesn't have value");
                let ty = info.ty;
                builder.emit(Store(ty, dst, rhs_val));
            }
            TirStmt::While { cond, body } => {
                let function_name = builder.get_current_function();

                let cond_block = builder.create_blockn(function_name, "loopcond");
                let body_block = builder.create_blockn(function_name, "loopbody");
                let end_block = builder.create_blockn(function_name, "loopend");

                // Finish current block, add cond as successor
                builder.emit(Jmp(cond_block));
                builder.add_successors(&[cond_block]);

                // Lower the cond expr
                builder.set_insert_point(cond_block);
                let cond_val = self.lower_expr(builder, cond);

                // Cond can jump to body and post-loop blocks
                let bool_val = Value::reg(builder.next_reg());
                builder.emit(Eq(
                    self.add_type(Type::Bool),
                    bool_val,
                    cond_val,
                    Value::imm(1),
                ));
                builder.emit(Br(bool_val, body_block, end_block));
                builder.add_successors(&[body_block, end_block]);

                // Lower the body stmt
                self.loop_labels.push((cond_block, end_block));
                builder.set_insert_point(body_block);
                self.lower_stmt(builder, body);
                self.loop_labels.pop();

                // Body always jumps to cond
                builder.emit(Jmp(cond_block));
                builder.add_successors(&[cond_block]);

                // Enter the post-loop block
                builder.set_insert_point(end_block);
            }
            TirStmt::Continue => {
                let Some((cond_block, _end_block)) = self.loop_labels.last() else {
                    die!("Continue statements can only be called within loops.");
                };
                builder.emit(Jmp(*cond_block));
            }
            TirStmt::Break => {
                let Some((_cond_block, end_block)) = self.loop_labels.last() else {
                    die!("Continue statements can only be called within loops.");
                };
                builder.emit(Jmp(*end_block));
            }
            TirStmt::If { cond, then_, else_ } => {
                let function_name = builder.get_current_function();

                let then_block = builder.create_blockn(function_name, "then");
                let else_block = builder.create_blockn(function_name, "else");
                let endif_block = builder.create_blockn(function_name, "endif");

                // Lower the cond expr
                let cond_val = self.lower_expr(builder, cond);

                // Cond can jump to either then or else
                let bool_val = Value::reg(builder.next_reg());
                builder.emit(Eq(
                    self.add_type(Type::Bool),
                    bool_val,
                    cond_val,
                    Value::imm(1),
                ));
                builder.emit(Br(bool_val, then_block, else_block));
                builder.add_successors(&[then_block, else_block]);

                // Lower then stmt
                builder.set_insert_point(then_block);
                self.lower_stmt(builder, then_);

                // Then will jump to endif if not already terminated
                if !builder.is_current_terminated() {
                    builder.emit(Jmp(endif_block));
                    builder.add_successors(&[endif_block]);
                }

                // Lower else stmt
                builder.set_insert_point(else_block);
                self.lower_stmt(builder, else_);

                // Else always jumps to join
                if !builder.is_current_terminated() {
                    builder.emit(Jmp(endif_block));
                    builder.add_successors(&[endif_block]);
                }

                // Enter the join block
                builder.set_insert_point(endif_block);
            }
            TirStmt::Return(Some(expr)) => {
                let ret_val = self.lower_expr(builder, expr);
                builder.emit(Ret(expr.ty, ret_val));
            }
            TirStmt::Return(None) => {
                builder.emit(Retv);
            }
            TirStmt::Block(stmts) => {
                for s in stmts {
                    self.lower_stmt(builder, s);
                }
            }
            TirStmt::Expr(expr) => {
                self.lower_expr(builder, expr);
            }
        }
    }

    fn lower_expr(&mut self, builder: &mut IRBuilder<LirInstr>, expr: &TirExpr) -> Value {
        match &expr.kind {
            TirExprKind::Void => todo!(),
            TirExprKind::Num(val) => Value::imm(*val),
            TirExprKind::Bool(val) => Value::imm((*val).into()),
            TirExprKind::Ident(symbol) => {
                let info = self.lookup_symbol(*symbol);
                let val = info.value.expect("Symbol doesn't have value");
                if val.is_mem() {
                    let dst = Value::reg(builder.next_reg());
                    builder.emit(Load(info.ty, dst, val));
                    dst
                } else {
                    val
                }
            }
            TirExprKind::Assign { lhs, rhs } => match &lhs.kind {
                TirExprKind::Ident(symbol) => {
                    let rhs_val = self.lower_expr(builder, rhs);
                    let info = self.lookup_symbol(*symbol);
                    let lhs_val = info.value.unwrap();
                    match info.kind {
                        SymbolKind::Local => builder.emit(Store(info.ty, lhs_val, rhs_val)),
                        SymbolKind::Arg(_) => builder.emit(Copy(lhs_val, rhs_val)),
                        _ => panic!(),
                    };
                    lhs_val
                }
                TirExprKind::Deref { target } => {
                    let target_val = self.lower_expr(builder, &target);
                    let rhs_val = self.lower_expr(builder, rhs);
                    builder.emit(Store(target.ty.get_pointee(), target_val, rhs_val));
                    rhs_val
                }
                _ => {
                    let lhs_val = self.lower_expr(builder, lhs);
                    let rhs_val = self.lower_expr(builder, rhs);
                    builder.emit(Copy(lhs_val, rhs_val));
                    lhs_val
                }
            },
            TirExprKind::AddrOf { expr } => {
                let dst = self.lower_expr(builder, &expr);
                if !dst.is_mem() {
                    die!("Address-of yielded non-address value: {dst}");
                }
                dst
            }
            TirExprKind::Deref { target } => {
                let target_val = self.lower_expr(builder, target);
                let dst = Value::reg(builder.next_reg());
                builder.emit(Load(target.ty.get_pointee(), dst, target_val));
                dst
            }
            TirExprKind::Un { op, rhs } => todo!(),
            TirExprKind::Bin { op, lhs, rhs } => {
                let ty = lhs.ty;
                let lhs_val = self.lower_expr(builder, &lhs);
                let rhs_val = self.lower_expr(builder, &rhs);
                match op {
                    BinOp::Add => {
                        let result = Value::reg(builder.next_reg());
                        builder.emit(Add(ty, result, lhs_val, rhs_val));
                        result
                    }
                    BinOp::Sub => {
                        let result = Value::reg(builder.next_reg());
                        builder.emit(Sub(ty, result, lhs_val, rhs_val));
                        result
                    }
                    BinOp::Mul => {
                        let result = Value::reg(builder.next_reg());
                        if lhs.ty.is_signed() {
                            builder.emit(Smul(ty, result, lhs_val, rhs_val));
                        } else {
                            builder.emit(Umul(ty, result, lhs_val, rhs_val));
                        }
                        result
                    }
                    BinOp::Div => {
                        let result = Value::reg(builder.next_reg());
                        if lhs.ty.is_signed() {
                            builder.emit(Sdiv(ty, result, lhs_val, rhs_val));
                        } else {
                            builder.emit(Udiv(ty, result, lhs_val, rhs_val));
                        }
                        result
                    }
                    BinOp::Eq => {
                        let result = Value::reg(builder.next_reg());
                        let ty = self.add_type(Type::Bool);
                        builder.emit(Eq(ty, result, lhs_val, rhs_val));
                        result
                    }
                    BinOp::Ne => {
                        let result = Value::reg(builder.next_reg());
                        let ty = self.add_type(Type::Bool);
                        builder.emit(Ne(ty, result, lhs_val, rhs_val));
                        result
                    }
                    BinOp::Le | BinOp::Lt | BinOp::Ge | BinOp::Gt => {
                        let result = Value::reg(builder.next_reg());
                        let ty = self.add_type(Type::Bool);
                        let (signed, unsigned) = match op {
                            BinOp::Lt => (
                                Slt(ty, result, lhs_val, rhs_val),
                                Ult(ty, result, lhs_val, rhs_val),
                            ),
                            BinOp::Le => (
                                Sle(ty, result, lhs_val, rhs_val),
                                Ule(ty, result, lhs_val, rhs_val),
                            ),
                            BinOp::Gt => (
                                Sgt(ty, result, lhs_val, rhs_val),
                                Ugt(ty, result, lhs_val, rhs_val),
                            ),
                            BinOp::Ge => (
                                Sge(ty, result, lhs_val, rhs_val),
                                Uge(ty, result, lhs_val, rhs_val),
                            ),
                            _ => unreachable!(),
                        };
                        if lhs.ty.is_signed() {
                            builder.emit(signed);
                        } else {
                            builder.emit(unsigned);
                        }
                        result
                    }
                }
            }
            TirExprKind::Cast { target_ty, expr } => {
                let rhs_val = self.lower_expr(builder, expr);
                let dst = Value::reg(builder.next_reg());
                if target_ty.bits() < expr.ty.bits() {
                    builder.emit(Trunc(*target_ty, dst, rhs_val))
                } else if target_ty.bits() > expr.ty.bits() {
                    // i8 -> i32: sext
                    // u8 -> u32: zext
                    // i8 -> u32: zext
                    // u8 -> i32: sext
                    if target_ty.is_signed() {
                        builder.emit(Sext(*target_ty, dst, rhs_val));
                    } else {
                        builder.emit(Zext(*target_ty, dst, rhs_val));
                    }
                }
                dst
            }
            TirExprKind::Call { callee, args } => todo!(),
        }
    }
}

// impl Compiler {
//     pub fn lower_func(
//         &mut self,
//         builder: &mut Builder<LirInstr>,
//         Spanned { inner: obj, span }: Spanned<TirObj>,
//     ) -> Builder<LirInstr> {
//         match obj {
//             TirObj::Fn {
//                 name,
//                 returns,
//                 args,
//                 body,
//             } => {
//                 let mut builder = Builder::new(self.func.raw_name.inner, 0, 0);
//                 let entry_bb = builder.new_bb("entry");
//                 builder.start_new_block(entry_bb);
//                 let symbol_table = std::mem::take(&mut self.global_symbols);
//                 for (name, info) in symbol_table.iter() {
//                     let name = *name;
//                     let SymbolInfo {
//                         raw_name,
//                         ty,
//                         kind,
//                         address_taken,
//                     } = *info;
//                     match kind {
//                         SymbolKind::Local => {
//                             let dst = LirVal::mem(builder.new_reg(), ty.size());
//                             builder.emit(Alloc(ty.into(), dst, name));
//                             self.func.var2val.insert(name, dst);
//                         }
//                         SymbolKind::Param(num) => {
//                             if address_taken {
//                                 let dst = LirVal::mem(builder.new_reg(), ty.size());
//                                 builder.emit(Sparam(ty.into(), dst, name, num));
//                                 self.func.var2val.insert(name, dst);
//                             } else {
//                                 let dst = LirVal::reg(builder.new_reg(), ty.size());
//                                 builder.emit(Param(ty.into(), dst, name, num));
//                                 self.func.var2val.insert(name, dst);
//                             }
//                         }
//                         SymbolKind::Global => todo!(),
//                         SymbolKind::Function => todo!(),
//                     }
//                 }
//
//                 let body_bb = builder.new_bb("body");
//                 builder.start_new_block(body_bb);
//                 self.lower_stmt(&mut builder, *body);
//
//                 if builder.get_terminator().is_none() {
//                     match returns.inner.lookup() {
//                         ResolvedType::Void => builder.buf.push(Retv),
//                         _ => die!(
//                             "Control flow reaches the end of a function that was expected to return {returns}"
//                         ),
//                     }
//                 }
//
//                 // This is done just to flush any instructions in the Basic Block buffer
//                 let exit_bb = builder.new_bb("");
//                 builder.start_new_block(exit_bb);
//
//                 // Do a final loop to compute each blocks' successors from its terminator
//                 // instruction
//                 for bb in builder.bbs.iter_mut() {
//                     match &bb.terminator {
//                         Br(ty, lir_val, tgt1, tgt2) => bb.succ.extend_from_slice(&[*tgt1, *tgt2]),
//                         Jmp(tgt) => bb.succ.push(*tgt),
//                         Retv => {}
//                         Ret(..) => {}
//                         x => die!("How did this end up as a terminator? {x}"),
//                     }
//                 }
//
//                 builder
//             }
//             _ => unreachable!(),
//         }
//     }
//
//     fn lower_stmt(
//         &mut self,
//         builder: &mut Builder<LirInstr>,
//         Spanned { inner: stmt, span }: Spanned<TirStmt>,
//     ) {
//         // TODO: Change this its kind of stupid. Span::content() should return just the source code
//         // slice, whereas Span::to_string() should print the file:row:col, the content, and the
//         // arrows
//         if CFG.verbose {
//             builder.emit(Comment(
//                 span.content().split('\n').nth(3).unwrap().to_string(),
//             ));
//         }
//         match stmt {
//             TirStmt::Let { ty, lhs, rhs } => {
//                 let rs1 = self.func.var2val.get(&lhs.inner).copied().unwrap();
//                 let LirValKind::Mem(..) = rs1.kind else {
//                     die!("Local variable must be an alloca'd pointer");
//                 };
//                 let ty = rhs.inner.ty;
//                 let rs2 = self.lower_expr(builder, rhs);
//                 builder.emit(Store(ty, rs1, rs2));
//             }
//             TirStmt::While { cond, body } => todo!(),
//             TirStmt::Continue => todo!(),
//             TirStmt::Break => todo!(),
//             TirStmt::If { cond, then_, else_ } => {
//                 // Create labels
//                 let if_bb = builder.new_bb("if");
//                 let then_bb = builder.new_bb("then");
//                 let else_bb = builder.new_bb("else");
//                 let end_bb = builder.new_bb("endif");
//
//                 // IF BB
//                 builder.start_new_block(if_bb);
//                 let cond_val = self.lower_expr(builder, cond);
//                 let branch = Br(
//                     self.resolved_types.add(ResolvedType::I8),
//                     cond_val,
//                     then_bb,
//                     else_bb,
//                 );
//                 builder.emit(branch);
//
//                 // THEN BB
//                 builder.start_new_block(then_bb);
//                 self.lower_stmt(builder, *then_);
//                 let jmp_end = Jmp(end_bb);
//                 builder.emit(jmp_end);
//                 let then_term = builder.get_terminator();
//
//                 // ELSE BB
//                 builder.start_new_block(else_bb);
//                 self.lower_stmt(builder, *else_);
//                 let jmp_end = Jmp(end_bb);
//                 builder.emit(jmp_end);
//                 let else_term = builder.get_terminator();
//
//                 // END BB (empty)
//                 match (then_term, else_term) {
//                     (Some(t), Some(e)) if is_ret(&t) && is_ret(&e) => {}
//                     _ => builder.start_new_block(end_bb),
//                 }
//             }
//             TirStmt::Return(spanned) => {
//                 if *spanned.inner.ty == ResolvedType::Void {
//                     builder.emit(Retv);
//                 } else {
//                     let ty = spanned.inner.ty;
//                     let rs1 = self.lower_expr(builder, spanned);
//                     builder.emit(Ret(ty, rs1));
//                 }
//             }
//             TirStmt::Block(spanneds) => {
//                 for s in spanneds {
//                     self.lower_stmt(builder, s);
//                 }
//             }
//             TirStmt::Expr(e) => {
//                 self.lower_expr(builder, e);
//             }
//         }
//     }
//
//     fn lower_expr(
//         &mut self,
//         builder: &mut Builder<LirInstr>,
//         Spanned { inner: expr, span }: Spanned<TirExpr>,
//     ) -> LirVal {
//         let ty = expr.ty;
//         let size = ty.size();
//         let dst = LirVal::reg(builder.new_reg(), size);
//         match expr.kind {
//             TirExprKind::Void => dst,
//             TirExprKind::Num(imm) => LirVal::imm(imm, size),
//             TirExprKind::Bool(b) => {
//                 let imm = b as i128;
//                 LirVal::imm(imm, size)
//             }
//             TirExprKind::Ident(varname) => {
//                 let Some(rs1) = self.func.var2val.get(&varname).copied() else {
//                     die!("Undefined variable: {varname}");
//                 };
//                 match rs1.kind {
//                     LirValKind::Mem(..) => {
//                         builder.emit(Load(ty, dst, rs1));
//                         dst
//                     }
//                     _ => rs1,
//                 }
//             }
//             TirExprKind::Un { op, rhs } => {
//                 let rs1 = self.lower_expr(builder, *rhs);
//                 match op {
//                     UnOp::Not => todo!(),
//                     UnOp::Neg => {
//                         builder.emit(Smul(ty, dst, rs1, LirVal::imm(-1, size)));
//                     }
//                 }
//                 dst
//             }
//             TirExprKind::Bin { op, lhs, rhs } => {
//                 let is_signed = lhs.inner.ty.is_signed();
//                 let rs1 = self.lower_expr(builder, *lhs);
//                 let rs2 = self.lower_expr(builder, *rhs);
//                 let instr = match (op, is_signed) {
//                     (BinOp::Add, _) => Add(ty, dst, rs1, rs2),
//                     (BinOp::Sub, _) => Sub(ty, dst, rs1, rs2),
//                     (BinOp::Mul, true) => Smul(ty, dst, rs1, rs2),
//                     (BinOp::Mul, false) => Umul(ty, dst, rs1, rs2),
//                     (BinOp::Div, true) => todo!(),
//                     (BinOp::Div, false) => todo!(),
//                     (BinOp::Eq, _) => Eq(ty, dst, rs1, rs2),
//                     (BinOp::Le, true) => Sle(ty, dst, rs1, rs2),
//                     (BinOp::Le, false) => Ule(ty, dst, rs1, rs2),
//                     (BinOp::Lt, true) => Slt(ty, dst, rs1, rs2),
//                     (BinOp::Lt, false) => Ult(ty, dst, rs1, rs2),
//                     (BinOp::Ge, true) => Sge(ty, dst, rs1, rs2),
//                     (BinOp::Ge, false) => Uge(ty, dst, rs1, rs2),
//                     (BinOp::Gt, true) => Sgt(ty, dst, rs1, rs2),
//                     (BinOp::Gt, false) => Ugt(ty, dst, rs1, rs2),
//                 };
//                 builder.emit(instr);
//                 dst
//             }
//             TirExprKind::Assign { lhs, rhs } => match lhs.inner.kind {
//                 TirExprKind::Ident(varname) => {
//                     let Some(rs1) = self.func.var2val.get(&varname).copied() else {
//                         die!("Lvar not found: {varname}");
//                     };
//                     let rs2 = self.lower_expr(builder, *rhs);
//                     match rs1.kind {
//                         LirValKind::Mem(..) => {
//                             builder.emit(Store(ty, rs1, rs2));
//                         }
//                         _ => {
//                             builder.emit(Copy(ty, rs1, rs2));
//                         }
//                     }
//                     rs2
//                 }
//                 TirExprKind::Deref { rhs: store_target } => {
//                     let rs1 = self.lower_expr(builder, *store_target);
//                     let rs2 = self.lower_expr(builder, *rhs);
//                     builder.emit(Store(ty, rs1, rs2));
//                     rs1
//                 }
//                 _ => unreachable!(),
//             },
//             TirExprKind::Deref { rhs } => {
//                 let rs1 = self.lower_expr(builder, *rhs);
//                 builder.emit(Load(ty, dst, rs1));
//                 dst
//             }
//             // AddrOf is a ty-instruction. It doesn't actually produce any "work" per-se.
//             // It simply grabs an existing pointer to the named storage and returns that for use by
//             // other expressions
//             TirExprKind::AddrOf { rhs } => {
//                 let TirExprKind::Ident(varname) = rhs.inner.kind else {
//                     unreachable!()
//                 };
//                 let Some(rs1) = self.func.var2val.get(&varname).copied() else {
//                     die!("Lvar not found: {varname}");
//                 };
//                 let LirValKind::Mem(..) = rs1.kind else {
//                     panic!("All named storage expressions should be allocated on the stack by now")
//                 };
//                 rs1
//             }
//             TirExprKind::Call { callee, args } => todo!(),
//             TirExprKind::Cast { target_ty, rhs } => {
//                 let ty = rhs.inner.ty;
//                 let rhs = self.lower_expr(builder, *rhs);
//                 if ty.size() == target_ty.inner.size() {
//                     // @T(T) is a No-op
//                     rhs
//                 } else {
//                     if ty.is_primitive() {
//                         if ty.size() < target_ty.inner.size() {
//                             if target_ty.inner.is_signed() {
//                                 builder.emit(Sext(target_ty.inner, dst, rhs));
//                             } else {
//                                 builder.emit(Zext(target_ty.inner, dst, rhs));
//                             }
//                         } else {
//                             builder.emit(Trunc(target_ty.inner, dst, rhs));
//                         }
//                     } else {
//                         todo!()
//                     }
//                     dst
//                 }
//             }
//             TirExprKind::Index { expr, index } => {
//                 todo!()
//             }
//             // The size of a type is known at checking time, so the checker literally replaces
//             // TirExprKind::SizeOf with Expr::Num, meaning this should never be hit
//             TirExprKind::SizeOfTy { .. } | TirExprKind::SizeOfExpr { .. } => unreachable!(),
//         }
//     }
// }
