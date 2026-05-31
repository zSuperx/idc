use std::collections::HashMap;

use crate::ast::BinOp;
use crate::backends::x86_64::{self, x86_64Function};
use crate::lir::{BB, Instr, LIRFunction, LirType, LirVal};
use crate::utils::align_n;

use x86_64::Instr::*;
use x86_64::Reg::*;
use x86_64::Val::*;

#[derive(Default)]
pub struct Emitter {
    v2h: HashMap<LirVal, x86_64::Val>,
    v_rsp: i128,
}

impl Emitter {
    pub fn new() -> Self {
        Self::default()
    }

    fn resolve_ptr(&self, v: LirVal, ty: LirType) -> x86_64::Val {
        match v {
            LirVal::Reg(id) => self.v2h.get(&v).copied().unwrap_or(Offset(ty, Virt(id), 0)),
            LirVal::Imm(_) => panic!("Resolve pointer called on immediate value"),
        }
    }

    fn resolve_val(&self, v: LirVal) -> x86_64::Val {
        match v {
            LirVal::Reg(id) => self.v2h.get(&v).copied().unwrap_or(Reg(Virt(id))),
            LirVal::Imm(i) => Imm(i),
        }
    }

    pub fn translate_func(&mut self, mut f: LIRFunction) -> x86_64Function {
        let mut buf = vec![];
        self.emit_prologue(&mut buf);
        let bbs = f.bbs.clone();
        let exit_bb = f.next_bb("");
        let mut bb_iter = bbs.iter().peekable();
        while let Some(bb) = bb_iter.next() {
            buf.push(Label(bb.name));
            for i in bb.instructions.iter() {
                // buf.push(Comment(format!("IR: {i}")));
                match *i {
                    Instr::Param(ty, dst, num, name) => {
                        let loc = match num {
                            0 => Reg(Rdi),
                            1 => Reg(Rsi),
                            2 => Reg(Rdx),
                            3 => Reg(Rcx),
                            4 => Reg(R8),
                            5 => Reg(R9),
                            _ => Offset(ty, Rbp, num.saturating_sub(6) as i128 + 8),
                        };
                        buf.push(Comment(format!("{dst} -> {loc}")));
                        self.v2h.insert(dst, loc);
                    }
                    Instr::Alloc(ty, dst, name) => {
                        let size = align_n(ty.size() as i128, ty.alignment());
                        let loc = Offset(ty, Rbp, self.v_rsp - 8);
                        let aligned_size = align_n(size, 16);
                        self.v_rsp -= aligned_size;
                        buf.push(Sub(Reg(Rsp), Imm(aligned_size as i128)));
                        buf.push(Comment(format!("{dst} -> {loc}")));
                        self.v2h.insert(dst, loc);
                    }
                    Instr::Copy(ty, dst, rs1) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        buf.push(Mov(dst, rs1))
                    }
                    // mov ..., [rs1]
                    Instr::Load(ty, dst, rs1) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_ptr(rs1, ty);

                        // I actually don't know if this will always be true
                        assert!(matches!(dst, Reg(_)));

                        let rs1 = match rs1 {
                            Reg(reg) => Offset(ty, reg, 0),
                            Offset(..) => rs1,
                            Imm(_) => unreachable!(),
                        };

                        buf.push(Mov(dst, rs1));
                    }
                    // lea [rs1], rs2
                    Instr::Store(ty, rs1, rs2) => {
                        let rs1 = self.resolve_ptr(rs1, ty);
                        let rs2 = self.resolve_val(rs2);

                        // I actually don't know if this will always be true
                        assert!(matches!(rs1, Offset(..)));

                        match rs2 {
                            Offset(..) => {
                                let tmp = self.resolve_val(f.next_reg());
                                buf.push(Lea(tmp, rs2));
                                buf.push(Mov(rs1, tmp));
                            }
                            _ => buf.push(Mov(rs1, rs2)),
                        }
                    }
                    Instr::Br(rs1, bb1, bb2) => {
                        let rs1 = self.resolve_val(rs1);
                        buf.push(Cmp(rs1, Imm(1)));
                        if bb_iter.peek().is_none_or(|next_bb| next_bb.name != bb1) {
                            buf.push(Jnz(bb1));
                        }
                        buf.push(Jz(bb2));
                    }
                    Instr::Jmp(bb) => buf.push(Jmp(bb)),
                    Instr::Add(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        buf.push(Mov(dst, rs1));
                        buf.push(Add(dst, rs2));
                    }
                    Instr::Sub(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        buf.push(Mov(dst, rs1));
                        buf.push(Sub(dst, rs2));
                    }
                    Instr::Muls(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        buf.push(Mov(dst, rs1));
                        buf.push(IMul(dst, rs2));
                    }
                    Instr::Mulu(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        buf.push(Mov(dst, rs1));
                        buf.push(Mul(dst, rs2));
                    }
                    Instr::Eq(ty, dst, rs1, rs2)
                    | Instr::Sgt(ty, dst, rs1, rs2)
                    | Instr::Sge(ty, dst, rs1, rs2)
                    | Instr::Slt(ty, dst, rs1, rs2)
                    | Instr::Sle(ty, dst, rs1, rs2)
                    | Instr::Ugt(ty, dst, rs1, rs2)
                    | Instr::Uge(ty, dst, rs1, rs2)
                    | Instr::Ult(ty, dst, rs1, rs2)
                    | Instr::Ule(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        buf.push(Cmp(rs1, rs2));
                        let tmp = self.resolve_val(f.next_reg());
                        buf.push(Mov(tmp, Imm(1)));
                        let cmov_instr = match i {
                            Instr::Eq(..) => Cmove(dst, tmp),
                            Instr::Sgt(..) => Cmovg(dst, tmp),
                            Instr::Sge(..) => Cmovge(dst, tmp),
                            Instr::Slt(..) => Cmovl(dst, tmp),
                            Instr::Sle(..) => Cmovle(dst, tmp),
                            Instr::Ugt(..) => Cmovg(dst, tmp),
                            Instr::Uge(..) => Cmovge(dst, tmp),
                            Instr::Ult(..) => Cmovl(dst, tmp),
                            Instr::Ule(..) => Cmovle(dst, tmp),
                            _ => unreachable!(),
                        };
                        buf.push(cmov_instr);
                    }

                    Instr::Ret(ty, rs1) => {
                        let rs1 = self.resolve_val(rs1);
                        buf.push(Mov(Reg(Rax), rs1));
                        buf.push(Jmp(exit_bb));
                    }

                    Instr::RetVoid => {
                        buf.push(Jmp(exit_bb));
                    }
                }
            }
        }
        buf.push(Label(exit_bb));
        self.emit_epilogue(&mut buf);
        x86_64Function::new(f.raw_name, buf)
    }

    fn emit_prologue(&mut self, buf: &mut Vec<x86_64::Instr>) {
        buf.push(Push(Reg(Rbp)));
        buf.push(Mov(Reg(Rbp), Reg(Rsp)));
    }

    fn emit_epilogue(&mut self, buf: &mut Vec<x86_64::Instr>) {
        buf.push(Mov(Reg(Rsp), Reg(Rbp)));
        buf.push(Pop(Reg(Rbp)));
        buf.push(Ret);
    }

    pub fn allocate_registers(&mut self, instructions: Vec<x86_64::Instr>) -> Vec<x86_64::Instr> {
        todo!()
    }
}
