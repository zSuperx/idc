use crate::IRs::tir::*;
use crate::ast::{BinOp, Type};
use crate::die;
use crate::state::{Function, SymbolKind, add_type};

use IRInstr::*;
use stir::isa::{Cmp, IRInstr, IRType, IRValue};
use stir::{builder::*, comment};

impl Function {
    pub fn codegen_func(&mut self, builder: &mut IRBuilder<IRInstr>) {
        let function_name = self.name.inner;

        builder.create_function(function_name);

        for (sym, info) in self.symbol_table.iter_mut() {
            let val = match info.kind {
                SymbolKind::Local => {
                    let dst = IRValue::ptr(builder.next_reg());
                    comment!(builder, "{sym} -> {dst}");
                    builder.emit(Alloca(info.ty.toIRType(), dst));
                    dst
                }
                SymbolKind::Arg(i) => {
                    let dst = IRValue::ptr(builder.next_reg());
                    comment!(builder, "{sym} -> {dst}");
                    builder.emit(Alloca(info.ty.toIRType(), dst));
                    dst
                }
                _ => panic!("What"),
            };

            info.value = Some(val);
        }

        let body = self.node.clone().unwrap();
        self.codegen_stmt(builder, &body);
        let default_return_instr = if *self.return_type.lookup() == Type::Void {
            Some(Retv)
        } else {
            None
        };
        let f = builder.get_current_function();
        if !f.verify((*self.return_type == Type::Void).then_some(Retv)) {
            die!(
                "Function doesn't return a value on some paths, but is expected to return {}: {}",
                self.return_type,
                self.name
            );
        }
    }

    fn codegen_stmt(&mut self, builder: &mut IRBuilder<IRInstr>, stmt: &TirStmt) {
        match stmt {
            TirStmt::Let { lhs, ty, rhs } => {
                let rhs_val = self.codegen_expr(builder, rhs);
                let info = self.lookup_symbol(*lhs);
                let dst = info.value.expect("Symbol doesn't have value");
                let ty = info.ty;
                builder.emit(Store(ty.toIRType(), dst, rhs_val));
            }
            TirStmt::While { cond, body } => {
                let function_name = builder.get_current_function_name();

                let cond_block = builder.newNamedBlock(function_name, "loopcond");
                let body_block = builder.newNamedBlock(function_name, "loopbody");
                let end_block = builder.newNamedBlock(function_name, "loopend");

                // Finish current block, add cond as successor
                builder.emit(Jmp(cond_block));
                builder.add_successors(&[cond_block]);

                // Codegen the cond expr
                builder.set_insert_point(cond_block);
                let cond_val = self.codegen_expr(builder, cond);

                // Cond can jump to body and post-loop blocks
                let bool_val = IRValue::reg(builder.next_reg());
                builder.emit(Icmp(
                    Cmp::Eq,
                    IRType::I1,
                    bool_val,
                    cond_val,
                    IRValue::imm(1),
                ));
                builder.emit(Br(bool_val, body_block, end_block));
                builder.add_successors(&[body_block, end_block]);

                // Codegen the body stmt
                self.loop_labels.push((cond_block, end_block));
                builder.set_insert_point(body_block);
                self.codegen_stmt(builder, body);
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
                let function_name = builder.get_current_function_name();

                let then_block = builder.newNamedBlock(function_name, "then");
                let else_block = builder.newNamedBlock(function_name, "else");
                let endif_block = builder.newNamedBlock(function_name, "endif");

                // Codegen the cond expr
                let cond_val = self.codegen_expr(builder, cond);

                // Cond can jump to either then or else
                let bool_val = IRValue::reg(builder.next_reg());
                builder.emit(Icmp(
                    Cmp::Eq,
                    IRType::I1,
                    bool_val,
                    cond_val,
                    IRValue::imm(1),
                ));
                builder.emit(Br(bool_val, then_block, else_block));
                builder.add_successors(&[then_block, else_block]);

                // Codegen then stmt
                builder.set_insert_point(then_block);
                self.codegen_stmt(builder, then_);

                // Then will jump to endif if not already terminated
                if !builder.is_current_terminated() {
                    builder.emit(Jmp(endif_block));
                    builder.add_successors(&[endif_block]);
                }

                // Codegen else stmt
                builder.set_insert_point(else_block);
                self.codegen_stmt(builder, else_);

                // Else always jumps to join
                if !builder.is_current_terminated() {
                    builder.emit(Jmp(endif_block));
                    builder.add_successors(&[endif_block]);
                }

                // Enter the join block
                builder.set_insert_point(endif_block);
            }
            TirStmt::Return(Some(expr)) => {
                let ret_val = self.codegen_expr(builder, expr);
                builder.emit(Ret(expr.ty.toIRType(), ret_val));
            }
            TirStmt::Return(None) => {
                builder.emit(Retv);
            }
            TirStmt::Block(stmts) => {
                for s in stmts {
                    self.codegen_stmt(builder, s);
                }
            }
            TirStmt::Expr(expr) => {
                self.codegen_expr(builder, expr);
            }
        }
    }

    fn codegen_expr(&mut self, builder: &mut IRBuilder<IRInstr>, expr: &TirExpr) -> IRValue {
        match &expr.kind {
            TirExprKind::Void => panic!("Can't codegen `void` value"),
            TirExprKind::Num(val) => IRValue::imm(*val),
            TirExprKind::Bool(val) => IRValue::imm((*val).into()),
            TirExprKind::ValueOf(symbol) => {
                let info = self.lookup_symbol(*symbol);
                let irty = info.ty.toIRType();
                let ptr = info.value.expect("Symbol doesn't have value");
                assert!(ptr.is_mem());
                let val = IRValue::from_type(builder.next_reg(), irty);
                builder.emit(Load(irty, ptr, val));
                val
            }
            TirExprKind::AddrOf(symbol) => {
                let info = self.lookup_symbol(*symbol);
                let ptr = info.value.expect("Symbol doesn't have value");
                assert!(ptr.is_mem());
                ptr
            }
            TirExprKind::Load { inner } => {
                let ptr = self.codegen_expr(builder, inner);
                assert!(ptr.is_mem());
                let irty = inner.ty.get_pointee().toIRType();
                let dst = IRValue::from_type(builder.next_reg(), irty);
                builder.emit(Load(irty, ptr, dst));
                dst
            }
            TirExprKind::Store { ptr, val } => {
                let ptr = self.codegen_expr(builder, ptr);
                assert!(ptr.is_mem());
                let irty = val.ty.toIRType();
                let val = self.codegen_expr(builder, val);
                builder.emit(Store(irty, ptr, val));
                val
            }
            TirExprKind::Bin { op, lhs, rhs } => {
                let ty = expr.ty;
                let irty = ty.toIRType();
                let lhs_val = self.codegen_expr(builder, &lhs);
                let rhs_val = self.codegen_expr(builder, &rhs);
                match op {
                    BinOp::Add => {
                        let result = IRValue::from_type(builder.next_reg(), irty);
                        builder.emit(Add(irty, result, lhs_val, rhs_val));
                        result
                    }
                    BinOp::Sub => {
                        let result = IRValue::reg(builder.next_reg());
                        builder.emit(Sub(ty.toIRType(), result, lhs_val, rhs_val));
                        result
                    }
                    BinOp::PtrAdd => {
                        let dst = IRValue::from_type(builder.next_reg(), irty);
                        if lhs.ty.is_pointer() {
                            let base_ty = lhs.ty.get_pointee().toIRType();
                            builder.emit(Getaddr(dst, lhs_val, base_ty, rhs_val));
                        } else {
                            let base_ty = rhs.ty.get_pointee().toIRType();
                            builder.emit(Getaddr(dst, rhs_val, base_ty, lhs_val));
                        }
                        dst
                    }
                    BinOp::PtrSub => todo!(),
                    BinOp::Mul => {
                        let result = IRValue::reg(builder.next_reg());
                        if lhs.ty.is_signed() {
                            builder.emit(Smul(ty.toIRType(), result, lhs_val, rhs_val));
                        } else {
                            builder.emit(Umul(ty.toIRType(), result, lhs_val, rhs_val));
                        }
                        result
                    }
                    BinOp::Div => {
                        let result = IRValue::reg(builder.next_reg());
                        if lhs.ty.is_signed() {
                            builder.emit(Sdiv(ty.toIRType(), result, lhs_val, rhs_val));
                        } else {
                            builder.emit(Udiv(ty.toIRType(), result, lhs_val, rhs_val));
                        }
                        result
                    }
                    BinOp::Eq => {
                        let result = IRValue::reg(builder.next_reg());
                        let ty = add_type(Type::Bool);
                        builder.emit(Icmp(Cmp::Eq, ty.toIRType(), result, lhs_val, rhs_val));
                        result
                    }
                    BinOp::Ne => {
                        let result = IRValue::reg(builder.next_reg());
                        let ty = add_type(Type::Bool);
                        builder.emit(Icmp(Cmp::Ne, ty.toIRType(), result, lhs_val, rhs_val));
                        result
                    }
                    BinOp::Le | BinOp::Lt | BinOp::Ge | BinOp::Gt => {
                        let result = IRValue::reg(builder.next_reg());
                        let ty = add_type(Type::Bool);
                        let (signed, unsigned) = match op {
                            BinOp::Lt => (
                                Icmp(Cmp::Slt, ty.toIRType(), result, lhs_val, rhs_val),
                                Icmp(Cmp::Ult, ty.toIRType(), result, lhs_val, rhs_val),
                            ),
                            BinOp::Le => (
                                Icmp(Cmp::Sle, ty.toIRType(), result, lhs_val, rhs_val),
                                Icmp(Cmp::Ule, ty.toIRType(), result, lhs_val, rhs_val),
                            ),
                            BinOp::Gt => (
                                Icmp(Cmp::Sgt, ty.toIRType(), result, lhs_val, rhs_val),
                                Icmp(Cmp::Ugt, ty.toIRType(), result, lhs_val, rhs_val),
                            ),
                            BinOp::Ge => (
                                Icmp(Cmp::Sge, ty.toIRType(), result, lhs_val, rhs_val),
                                Icmp(Cmp::Uge, ty.toIRType(), result, lhs_val, rhs_val),
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
            TirExprKind::Cast {
                target_ty,
                expr: inner,
            } => {
                let rhs_val = self.codegen_expr(builder, inner);
                let from_ty = inner.ty.toIRType();
                let to_ty = expr.ty.toIRType();
                comment!(builder, "Casting to {to_ty}");
                let dst = IRValue::from_type(builder.next_reg(), to_ty);
                if target_ty.bits() < inner.ty.bits() {
                    builder.emit(Trunc(to_ty, dst, from_ty, rhs_val))
                } else if target_ty.bits() > inner.ty.bits() {
                    // i8 -> i32: sext
                    // u8 -> u32: zext
                    // i8 -> u32: zext
                    // u8 -> i32: sext
                    if target_ty.is_signed() {
                        builder.emit(Sext(to_ty, dst, from_ty, rhs_val));
                    } else {
                        builder.emit(Zext(to_ty, dst, from_ty, rhs_val));
                    }
                } else {
                    builder.emit(Copy(to_ty, dst, rhs_val));
                }
                dst
            }
            x => todo!("Codegen {x:?}"),
        }
    }
}
