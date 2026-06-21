// This file is responsible for translating LIR to x86 MIR and then performing register
// live-analysis and register allocation.
//
// Live analysis is based off the algorithm described here:
// https://en.wikipedia.org/wiki/Live-variable_analysis
use std::collections::HashMap;

use crate::arch::lir::{LirVal, LirValKind};
use crate::arch::x86::*;
use crate::autogen::LirInstr;
use crate::autogen::x86Instr;
use crate::prelude::*;

use bitset::BitSet;
use heuristic_graph_coloring::{VecVecGraph, color_greedy_by_degree};
use x86Instr::*;
use x86Reg::*;
use x86ValKind::*;

const RBP: x86Val = x86Val::reg(BP as usize, 8);
const RSP: x86Val = x86Val::reg(SP as usize, 8);
const RAX: x86Val = x86Val::reg(A as usize, 8);

#[derive(Default)]
pub struct Emitter {
    v2p: HashMap<LirVal, x86Val>,
    v_rsp: i128,
}

impl Emitter {
    pub fn new() -> Self {
        Self::default()
    }

    fn resolve_ptr(&self, v: LirVal) -> x86Val {
        match v.kind {
            LirValKind::Reg(_) => panic!("Resolve pointer called on raw register"),
            LirValKind::Mem(reg, _) => self
                .v2p
                .get(&v)
                .copied()
                .expect("Could not find pointer in v2p map"),
            // .unwrap_or(x86Val::mem(reg, 0, v.size)),
            LirValKind::Imm(_) => panic!("Resolve pointer called on immediate value"),
        }
    }

    fn resolve_val(&self, v: LirVal, builder: &mut Builder<x86Instr>) -> x86Val {
        match v.kind {
            LirValKind::Reg(id) => self.v2p.get(&v).copied().expect("Hello"),
            // unwrap_or(x86Val::reg(id, v.size)),
            LirValKind::Mem(..) => {
                let ptr = self.resolve_ptr(v);
                let reg = x86Val::reg(builder.next_reg(), 8);
                builder.emit(Lea(reg, ptr));
                reg
            }
            LirValKind::Imm(i) => x86Val::imm(i, v.size),
        }
    }

    pub fn translate_func(&mut self, f: Builder<LirInstr>) -> Builder<x86Instr> {
        let mut builder = Builder::new(f.name, f.bb_count, f.vreg_count);

        let prologue = builder.next_bb("prologue");
        builder.start_new_block(prologue);
        builder.emit(Push(RBP));
        builder.emit(Mov(RBP, RSP));

        let epilogue = builder.next_bb("epilogue");

        let mut bb_iter = f.bbs.iter().peekable();
        while let Some(bb) = bb_iter.next() {
            builder.start_new_block(bb.name);
            for i in bb.instructions.iter() {
                match *i {
                    LirInstr::Param(dst, name, num) => {
                        let loc = match num {
                            0 => x86Val::reg(DI as usize, dst.size),
                            1 => x86Val::reg(SI as usize, dst.size),
                            2 => x86Val::reg(D as usize, dst.size),
                            3 => x86Val::reg(C as usize, dst.size),
                            4 => x86Val::reg(R8 as usize, dst.size),
                            5 => x86Val::reg(R9 as usize, dst.size),
                            _ => x86Val::mem(
                                BP as usize,
                                num.saturating_sub(6) as i128 + 8,
                                dst.size,
                            ),
                        };
                        builder.emit(Comment(format!("{dst} -> {loc}")));
                        self.v2p.insert(dst, loc);
                    }
                    LirInstr::Alloc(dst, name) => {
                        let loc = x86Val::mem(BP as usize, self.v_rsp - 8, dst.size);
                        // TODO: alignment correction should be done after an "sroa" pass i think
                        let aligned_size = align_n(dst.size as i128, 16);
                        self.v_rsp -= aligned_size;
                        builder.emit(Sub(RSP, x86Val::imm(aligned_size as i128, 8)));
                        builder.emit(Comment(format!("{dst} -> {loc}")));
                        self.v2p.insert(dst, loc);
                    }
                    LirInstr::Copy(dst, rs1) => {
                        let dst = self.resolve_val(dst, &mut builder);
                        let rs1 = self.resolve_val(rs1, &mut builder);
                        builder.emit(Mov(dst, rs1))
                    }
                    // mov ..., [rs1]
                    LirInstr::Load(dst, rs1) => {
                        let dst = self.resolve_val(dst, &mut builder);
                        let rs1 = self.resolve_ptr(rs1);

                        // I actually don't know if this will always be true
                        assert!(matches!(dst.kind, Reg(..)));

                        let rs1 = match rs1.kind {
                            Reg(reg) => x86Val::mem(reg, 0, rs1.size),
                            Mem(..) => rs1,
                            Imm(_) => unreachable!(),
                        };

                        builder.emit(Mov(dst, rs1));
                    }
                    // lea [rs1], rs2
                    LirInstr::Store(rs1, rs2) => {
                        let rs1 = self.resolve_ptr(rs1);
                        let rs2 = self.resolve_val(rs2, &mut builder);

                        // I actually don't know if this will always be true
                        assert!(matches!(rs1.kind, Mem(..)));

                        match rs2.kind {
                            Mem(..) => {
                                let tmp =
                                    self.resolve_val(LirVal::reg(builder.next_reg(), rs2.size), &mut builder);
                                builder.emit(Lea(tmp, rs2));
                                builder.emit(Mov(rs1, tmp));
                            }
                            _ => builder.emit(Mov(rs1, rs2)),
                        }
                    }
                    LirInstr::Br(rs1, bb1, bb2) => {
                        let rs1 = self.resolve_val(rs1, &mut builder);
                        builder.emit(Cmp(rs1, x86Val::imm(1, rs1.size)));
                        // If we fall through to the "then" block, there's no need to emit a `jnz`
                        if bb_iter.peek().is_none_or(|next_bb| next_bb.name != bb1) {
                            builder.emit(Jnz(bb1));
                        }
                        builder.emit(Jz(bb2));
                    }
                    LirInstr::Jmp(bb) => builder.emit(Jmp(bb)),
                    LirInstr::Add(dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst, &mut builder);
                        let rs1 = self.resolve_val(rs1, &mut builder);
                        let rs2 = self.resolve_val(rs2, &mut builder);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(Add(dst, rs2));
                    }
                    LirInstr::Sub(dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst, &mut builder);
                        let rs1 = self.resolve_val(rs1, &mut builder);
                        let rs2 = self.resolve_val(rs2, &mut builder);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(Sub(dst, rs2));
                    }
                    LirInstr::Smul(dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst, &mut builder);
                        let rs1 = self.resolve_val(rs1, &mut builder);
                        let rs2 = self.resolve_val(rs2, &mut builder);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(Imul(dst, rs2));
                    }
                    LirInstr::Umul(dst, rs1, rs2) => {
                        // let dst = RAX;
                        // let rs1 = self.resolve_val(rs1, &mut builder);
                        // let rs2 = self.resolve_reg(rs2, &mut builder);
                        // builder.emit(Mov(dst, rs1));
                        // builder.emit(Mul(dst, rs2));
                        todo!("Implement unsigned multiply (mul)");
                    }
                    LirInstr::Eq(dst, rs1, rs2)
                    | LirInstr::Sgt(dst, rs1, rs2)
                    | LirInstr::Sge(dst, rs1, rs2)
                    | LirInstr::Slt(dst, rs1, rs2)
                    | LirInstr::Sle(dst, rs1, rs2)
                    | LirInstr::Ugt(dst, rs1, rs2)
                    | LirInstr::Uge(dst, rs1, rs2)
                    | LirInstr::Ult(dst, rs1, rs2)
                    | LirInstr::Ule(dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst, &mut builder);
                        let rs1 = self.resolve_val(rs1, &mut builder);
                        let rs2 = self.resolve_val(rs2, &mut builder);
                        let tmp = self.resolve_val(LirVal::reg(builder.next_reg(), dst.size), &mut builder);
                        builder.emit(Mov(tmp, x86Val::imm(0, tmp.size)));
                        builder.emit(Cmp(rs1, rs2));
                        builder.emit(Mov(dst, x86Val::imm(1, tmp.size)));
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

                    LirInstr::Ret(rs1) => {
                        println!("ret {rs1}");
                        let rs1 = self.resolve_val(rs1, &mut builder);
                        println!("Becomes ret {rs1}");
                        builder.emit(Mov(x86Val::reg(A as usize, rs1.size), rs1));
                        builder.emit(Jmp(epilogue));
                    }

                    LirInstr::Retv => {
                        builder.emit(Jmp(epilogue));
                    }
                    LirInstr::Udiv(lir_val, lir_val1, lir_val2) => todo!(),
                    LirInstr::Sdiv(lir_val, lir_val1, lir_val2) => todo!(),
                }
            }
        }
        builder.start_new_block(epilogue);
        builder.emit(Mov(RSP, RBP));
        builder.emit(Pop(RBP));
        builder.emit(Ret);

        let exit_bb = builder.next_bb("");
        builder.start_new_block(exit_bb);

        let mut p = builder.bbs.iter_mut().peekable();
        while let Some(bb) = p.next() {
            for i in bb.instructions.iter() {
                match i {
                    Jmp(tgt) | Je(tgt) | Jne(tgt) | Jl(tgt) | Jle(tgt) | Jg(tgt) | Jge(tgt)
                    | Jo(tgt) | Jno(tgt) | Jz(tgt) | Jnz(tgt) => {
                        bb.succ.push(*tgt);
                        p.peek().inspect(|next| {
                            if next.name != *tgt {
                                bb.succ.push(next.name)
                            }
                        });
                    }
                    _ => {}
                }
            }
        }

        self.allocate_registers(&mut builder);

        builder
    }

    pub fn allocate_registers(&mut self, builder: &mut Builder<x86Instr>) {
        let total_regs = builder.vreg_count + 16;

        let mut live_in = vec![BitSet::with_size(total_regs); builder.bbs.len()];
        let mut live_out = vec![BitSet::with_size(total_regs); builder.bbs.len()];
        let mut def = vec![BitSet::with_size(total_regs); builder.bbs.len()];
        let mut use_ = vec![BitSet::with_size(total_regs); builder.bbs.len()];
        // This just maps BB -> usize, so we have reverse index from BB to its place in the builder
        // bb list
        let mut map_index = HashMap::new();

        // Build the USE & DEF sets (also called GEN & KILL)
        for (bbid, bb) in builder.bbs.iter_mut().enumerate() {
            map_index.insert(bb.name, bbid);
            for i in bb.instructions.iter_mut() {
                for src in i.srcs() {
                    match src.kind {
                        Reg(reg) | Mem(reg, _) => {
                            if !def[bbid].contains(reg) {
                                use_[bbid].insert(reg);
                            }
                        }
                        _ => {}
                    }
                }

                for dst in i.dsts() {
                    match dst.kind {
                        Reg(reg) | Mem(reg, _) => {
                            def[bbid].insert(reg);
                        }
                        _ => {}
                    }
                }
            }
        }

        // TODO: This is a suboptimal convergence algorithm to compute the LIVE_{IN,OUT} sets.
        // It can be vastly improved by popping items out of a worklist. When a basic block sees
        // its LIVE sets change, it should push its predecessors into_usize the worklist.
        //
        // The issue is that there is currently no way to find a basic block's predecessors. This is
        // a TODO for when that API gets overhauled. For now, just loop forever until convergence.
        loop {
            let mut changed = false;

            for bb in builder.bbs.iter().rev() {
                let index = map_index.get(&bb.name).copied().unwrap();
                let basicblock = &builder.bbs[index];

                let live_in0 = live_in[index].clone();
                let live_out0 = live_out[index].clone();

                for succ in basicblock.succ.iter() {
                    let succ_index = map_index.get(succ).copied().unwrap();
                    live_out[index] = live_out[index].union(&live_in[succ_index]);
                }

                live_in[index] = use_[index].union(&live_out[index].difference(&def[index]));

                if live_in0 != live_in[index] || live_out0 != live_out[index] {
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }

        let mut graph = VecVecGraph::new(builder.vreg_count + 16);

        for bb in builder.bbs.iter_mut() {
            let index = map_index[&bb.name];
            let mut live = &mut live_out[index];

            for i in bb.instructions.iter_mut().rev() {
                for dst in i.dsts() {
                    match dst.kind {
                        Reg(reg) | Mem(reg, _) => {
                            for v in live.iter() {
                                if reg != v {
                                    graph.add_edge(reg, v);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                for dst in i.dsts() {
                    match dst.kind {
                        Reg(reg) | Mem(reg, _) => {
                            live.remove(reg);
                        }
                        _ => {}
                    }
                }

                for src in i.srcs() {
                    match src.kind {
                        Reg(reg) | Mem(reg, _) => {
                            live.insert(reg);
                        }
                        _ => {}
                    }
                }
            }
        }

        for i in 0..16 {
            for j in i + 1..16 {
                graph.add_edge(i, j);
            }
        }

        let coloring = color_greedy_by_degree(&graph);

        for bb in builder.bbs.iter_mut() {
            // let new = vec![];
            for i in bb.instructions.iter_mut() {
                for dst in i.dsts() {
                    match &mut dst.kind {
                        Reg(reg) | Mem(reg, _) => {
                            if *reg > 15 {
                                *reg = coloring[*reg];
                            }
                        }
                        _ => {}
                    }
                }

                for src in i.srcs() {
                    match &mut src.kind {
                        Reg(reg) | Mem(reg, _) => {
                            if *reg > 15 {
                                *reg = coloring[*reg];
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
