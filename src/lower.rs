use crate::{ast::*, compiler::Compiler, lir::*, tir::*};

use Instr::*;

impl Compiler {
    fn next_reg(&mut self) -> RegId {
        let id = self.reg_count;
        self.reg_count += 1;
        RegId(id)
    }

    fn next_lbl(&mut self, name: Option<&'static str>) -> Lbl {
        let id = self.lbl_count;
        self.lbl_count += 1;
        Lbl(name.unwrap_or(".L"), id)
    }

    fn commit_bb(&mut self, name: Option<&'static str>, buf: &mut Vec<Instr>, terminator: Instr) {
        let id = self.bb_count;
        self.bb_count += 1;
        let bb = BasicBlock::new(name, id, std::mem::take(buf), terminator);
        self.bbs.push(bb);
    }

    pub fn lower_obj(&mut self, buf: &mut Vec<Instr>, obj: Spanned<TirObj>) -> Vec<BasicBlock> {
        match obj.inner.kind {
            TirObjKind::Fn {
                name,
                returns,
                args,
                body,
                lvars,
            } => {
                buf.push(Function(name.inner));
                for (lvar, ty) in lvars {
                    buf.push(Alloc(ty, lvar));
                }
                self.lower_stmt(buf, *body);
                self.commit_bb(None, buf, buf.last().copied().unwrap());
                std::mem::take(&mut self.bbs)
            }
            TirObjKind::Global { lhs, rhs } => todo!(),
            TirObjKind::Struct { name, fields } => todo!(),
        }
    }

    fn lower_stmt(&mut self, buf: &mut Vec<Instr>, stmt: Spanned<TirStmt>) {
        match stmt.inner.kind {
            TirStmtKind::Let { lhs, ty, rhs } => {
                let rhs_val = self.lower_expr(buf, rhs);
                buf.push(Write {
                    loc: lhs.inner,
                    rs1: rhs_val,
                });
            }
            TirStmtKind::While { cond, body } => todo!(),
            TirStmtKind::Continue => todo!(),
            TirStmtKind::Break => todo!(),
            TirStmtKind::If { cond, then_, else_ } => {
                let then_lbl = self.next_lbl(Some("then"));
                let else_lbl = self.next_lbl(Some("else"));
                let end_lbl = self.next_lbl(Some("endif"));

                let if_val = self.lower_expr(buf, cond);
                let term = Br {
                    rs1: if_val,
                    lbl1: then_lbl,
                    lbl2: else_lbl,
                };
                buf.push(term);
                self.commit_bb(None, buf, term);

                buf.push(Label(then_lbl));
                self.lower_stmt(buf, *then_);
                let term = Jmp { lbl: end_lbl };
                buf.push(term);
                self.commit_bb(None, buf, term);

                buf.push(Label(else_lbl));
                self.lower_stmt(buf, *else_);
                let term = Jmp { lbl: end_lbl };
                buf.push(term);
                self.commit_bb(None, buf, term);

                buf.push(Label(end_lbl));
            }
            TirStmtKind::Return(spanned) => todo!(),
            TirStmtKind::Block(spanneds) => {
                for s in spanneds {
                    self.lower_stmt(buf, s);
                }
            }
            TirStmtKind::Expr(e) => {
                self.lower_expr(buf, e);
            }
        }
    }

    fn lower_expr(&mut self, buf: &mut Vec<Instr>, expr: Spanned<TirExpr>) -> RegId {
        let dst = self.next_reg();
        match expr.inner.kind {
            TirExprKind::Num(imm) => {
                buf.push(Const { dst, imm });
                dst
            }
            TirExprKind::Bool(b) => {
                buf.push(Const {
                    dst,
                    imm: b as i128,
                });
                dst
            }
            TirExprKind::Ident(loc) => {
                buf.push(Read { dst, loc });
                dst
            }
            TirExprKind::Un { op, rhs } => {
                let rs1 = self.lower_expr(buf, *rhs);
                match op {
                    UnOp::Not => todo!(),
                    UnOp::Neg => {
                        let rs2 = self.next_reg();
                        buf.push(Const { dst: rs2, imm: -1 });
                        buf.push(Bin {
                            dst,
                            op: BinOp::Mul,
                            rs1,
                            rs2,
                        });
                    }
                }
                dst
            }
            TirExprKind::Bin { op, lhs, rhs } => {
                let rs1 = self.lower_expr(buf, *lhs);
                let rs2 = self.lower_expr(buf, *rhs);
                buf.push(Bin { dst, op, rs1, rs2 });
                dst
            }
            TirExprKind::Assign { lhs, rhs } => {
                todo!()
            }
            TirExprKind::Deref { rhs } => todo!(),
            TirExprKind::AddrOf { rhs } => todo!(),
            TirExprKind::Call { callee, args } => todo!(),
        }
    }
}
