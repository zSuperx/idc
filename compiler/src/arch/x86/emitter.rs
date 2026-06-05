// This file is responsible for translating LIR to x86 MIR and then performing register
// live-analysis and register allocation.
//
// Live analysis is based off the algorithm described here:
// https://en.wikipedia.org/wiki/Live-variable_analysis
use std::collections::{HashMap, HashSet};

use crate::arch::lir::{LirType, LirVal};
use crate::arch::x86::*;
use crate::autogen::LirInstr;
use crate::autogen::x86Instr;
use crate::prelude::*;

use bitset::BitSet;
use heuristic_graph_coloring::{VecVecGraph, color_greedy_by_degree};
use x86Instr::*;
use x86Reg::*;
use x86Val::*;

#[derive(Default)]
pub struct Emitter {
    v2h: HashMap<LirVal, x86Val>,
    v_rsp: i128,
}

impl Emitter {
    pub fn new() -> Self {
        Self::default()
    }

    fn resolve_ptr(&self, v: LirVal, ty: LirType) -> x86Val {
        match v {
            LirVal::Reg(id) => self.v2h.get(&v).copied().unwrap_or(Mem(ty, Virt(id), 0)),
            LirVal::Imm(_) => panic!("Resolve pointer called on immediate value"),
        }
    }

    fn resolve_val(&self, v: LirVal) -> x86Val {
        match v {
            LirVal::Reg(id) => self.v2h.get(&v).copied().unwrap_or(Reg(Virt(id))),
            LirVal::Imm(i) => Imm(i),
        }
    }

    pub fn translate_func(&mut self, f: Builder<LirInstr>) -> Builder<x86Instr> {
        let mut builder = Builder::new(f.name, f.bb_count, f.vreg_count);

        let prologue = builder.next_bb("prologue");
        builder.start_new_block(prologue);
        builder.emit(Push(Reg(Rbp)));
        builder.emit(Mov(Reg(Rbp), Reg(Rsp)));

        let epilogue = builder.next_bb("epilogue");

        let mut bb_iter = f.bbs.iter().peekable();
        while let Some(bb) = bb_iter.next() {
            builder.start_new_block(bb.name);
            for i in bb.instructions.iter() {
                match *i {
                    LirInstr::Param(ty, dst, name, num) => {
                        let loc = match num {
                            0 => Reg(Rdi),
                            1 => Reg(Rsi),
                            2 => Reg(Rdx),
                            3 => Reg(Rcx),
                            4 => Reg(R8),
                            5 => Reg(R9),
                            _ => Mem(ty, Rbp, num.saturating_sub(6) as i128 + 8),
                        };
                        builder.emit(Comment(format!("{dst} -> {loc}")));
                        self.v2h.insert(dst, loc);
                    }
                    LirInstr::Alloc(ty, dst, name) => {
                        let size = align_n(ty.size() as i128, ty.alignment());
                        let loc = Mem(ty, Rbp, self.v_rsp - 8);
                        let aligned_size = align_n(size, 16);
                        self.v_rsp -= aligned_size;
                        builder.emit(Sub(Reg(Rsp), Imm(aligned_size as i128)));
                        builder.emit(Comment(format!("{dst} -> {loc}")));
                        self.v2h.insert(dst, loc);
                    }
                    LirInstr::Copyr(ty, dst, rs1) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        builder.emit(Mov(dst, rs1))
                    }
                    // mov ..., [rs1]
                    LirInstr::Load(ty, dst, rs1) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_ptr(rs1, ty);

                        // I actually don't know if this will always be true
                        assert!(matches!(dst, Reg(_)));

                        let rs1 = match rs1 {
                            Reg(reg) => Mem(ty, reg, 0),
                            Mem(..) => rs1,
                            Imm(_) => unreachable!(),
                        };

                        builder.emit(Mov(dst, rs1));
                    }
                    // lea [rs1], rs2
                    LirInstr::Store(ty, rs1, rs2) => {
                        let rs1 = self.resolve_ptr(rs1, ty);
                        let rs2 = self.resolve_val(rs2);

                        // I actually don't know if this will always be true
                        assert!(matches!(rs1, Mem(..)));

                        match rs2 {
                            Mem(..) => {
                                let tmp = self.resolve_val(LirVal::Reg(builder.next_reg()));
                                builder.emit(Lea(tmp, rs2));
                                builder.emit(Mov(rs1, tmp));
                            }
                            _ => builder.emit(Mov(rs1, rs2)),
                        }
                    }
                    LirInstr::Br(rs1, bb1, bb2) => {
                        let rs1 = self.resolve_val(rs1);
                        builder.emit(Cmp(rs1, Imm(1)));
                        if bb_iter.peek().is_none_or(|next_bb| next_bb.name != bb1) {
                            builder.emit(Jnz(bb1));
                        }
                        builder.emit(Jz(bb2));
                    }
                    LirInstr::Jmp(bb) => builder.emit(Jmp(bb)),
                    LirInstr::Add(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(Add(dst, rs2));
                    }
                    LirInstr::Sub(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(Sub(dst, rs2));
                    }
                    LirInstr::Smul(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(Imul(dst, rs2));
                    }
                    LirInstr::Umul(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(Mul(dst, rs2));
                    }
                    LirInstr::Eq(ty, dst, rs1, rs2)
                    | LirInstr::Sgt(ty, dst, rs1, rs2)
                    | LirInstr::Sge(ty, dst, rs1, rs2)
                    | LirInstr::Slt(ty, dst, rs1, rs2)
                    | LirInstr::Sle(ty, dst, rs1, rs2)
                    | LirInstr::Ugt(ty, dst, rs1, rs2)
                    | LirInstr::Uge(ty, dst, rs1, rs2)
                    | LirInstr::Ult(ty, dst, rs1, rs2)
                    | LirInstr::Ule(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        builder.emit(Cmp(rs1, rs2));
                        let tmp = self.resolve_val(LirVal::Reg(builder.next_reg()));
                        builder.emit(Mov(tmp, Imm(1)));
                        let cmov_instr = match i {
                            LirInstr::Eq(..) => Cmove(dst, tmp),
                            LirInstr::Sgt(..) => Cmovg(dst, tmp),
                            LirInstr::Sge(..) => Cmovge(dst, tmp),
                            LirInstr::Slt(..) => Cmovl(dst, tmp),
                            LirInstr::Sle(..) => Cmovle(dst, tmp),
                            LirInstr::Ugt(..) => Cmovg(dst, tmp),
                            LirInstr::Uge(..) => Cmovge(dst, tmp),
                            LirInstr::Ult(..) => Cmovl(dst, tmp),
                            LirInstr::Ule(..) => Cmovle(dst, tmp),
                            _ => unreachable!(),
                        };
                        builder.emit(cmov_instr);
                    }

                    LirInstr::Ret(ty, rs1) => {
                        let rs1 = self.resolve_val(rs1);
                        builder.emit(Mov(Reg(Rax), rs1));
                        builder.emit(Jmp(epilogue));
                    }

                    LirInstr::Retv => {
                        builder.emit(Jmp(epilogue));
                    }
                    LirInstr::Udiv(lir_type, lir_val, lir_val1, lir_val2) => todo!(),
                    LirInstr::Sdiv(lir_type, lir_val, lir_val1, lir_val2) => todo!(),
                }
            }
        }
        builder.start_new_block(epilogue);
        builder.emit(Mov(Reg(Rsp), Reg(Rbp)));
        builder.emit(Pop(Reg(Rbp)));
        builder.emit(Ret);

        let exit_bb = builder.next_bb("");
        builder.start_new_block(exit_bb);

        self.allocate_registers(&builder);

        builder
    }

    pub fn allocate_registers(&mut self, builder: &Builder<x86Instr>) -> Vec<x86Instr> {
        let total_regs = builder.vreg_count + 16;

        let out = vec![];

        let mut live_in = vec![BitSet::with_size(total_regs); builder.bbs.len()];
        let mut live_out = vec![BitSet::with_size(total_regs); builder.bbs.len()];
        let mut def = vec![BitSet::with_size(total_regs); builder.bbs.len()];
        let mut use_ = vec![BitSet::with_size(total_regs); builder.bbs.len()];

        for (bbid, bb) in builder.bbs.iter().enumerate() {
            for i in bb.instructions.iter() {
                for src in i.srcs() {
                    match *src {
                        Reg(reg) | Mem(_, reg, _) => {
                            if !def[bbid].contains(reg.into()) {
                                use_[bbid].insert(reg.into());
                            }
                        }
                        Imm(_) => {}
                    }
                }

                for dst in i.dsts() {
                    match *dst {
                        Reg(reg) | Mem(_, reg, _) => {
                            def[bbid].insert(reg.into());
                        }
                        Imm(_) => {}
                    }
                }
            }
        }
        let mut worklist = vec![builder.bbs.len() - 1];
        let mut visited = HashSet::new();

        while let Some(bb_index) = worklist.pop() {
            if !visited.insert(bb_index) {
                continue;
            }

            let live_in0 = live_in[bb_index].clone();
            let live_out0 = live_out[bb_index].clone();
            // TODO: In order to proceed, we must have a way to get a particular block's successors
            // This is in theory not difficult to do since it requires simply observing the terminal
            // instruction and seeing where it branches. This, however, is architecture specific. A
            // better solution would be to modify gen.py to auto-generate a 
            //
            // `fn targets(&self) -> Vec<BB>`
            //
            // that we can use before this loop (or even in the lowering phase). Part of this
            // limitation is also because `Builder<I>` is generic over an instruction type `I`, and
            // its API does not give a way to encode successor blocks in the lowering phase. This is
            // another avenue that can be explored.

            let tmp = BitSet::new();
        }

        let phys = &[
            Rdi, Rsi, Rdx, Rcx, R8, R9, Rax, R10, R11, Rsp, Rbp, Rbx, R12, R13, R14, R15,
        ];
        let mut graph = VecVecGraph::new(builder.vreg_count + 16);

        for i in 0..phys.len() {
            for j in i + 1..phys.len() {
                graph.add_edge(phys[i].into(), phys[j].into());
            }
        }

        let coloring = color_greedy_by_degree(&graph);

        out
    }
}
