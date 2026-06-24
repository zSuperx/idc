// This file is responsible for translating LIR to x86 MIR and then performing register
// live-analysis and register allocation.
//
// Live analysis is based off the algorithm described here:
// https://en.wikipedia.org/wiki/Live-variable_analysis
use std::collections::HashMap;

use crate::arch::lir::*;
use crate::arch::x86::*;
use crate::ast::*;
use crate::{CFG, prelude::*};

use bitset::BitSet;
use heuristic_graph_coloring::{VecVecGraph, color_greedy_by_degree};
use x86Instr::*;
use x86ValKind::*;

const RBP: x86Val = x86Val::reg(BP, 8);
const RSP: x86Val = x86Val::reg(SP, 8);
const RAX: x86Val = x86Val::reg(A, 8);

#[derive(Default)]
pub struct Emitter {
    v2p: HashMap<LirVal, x86Val>,
    v_rsp: i128,
}

impl Emitter {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_mem(&self, ty: &RealType, v: LirVal) -> x86Val {
        let ret = match v.kind {
            LirValKind::Reg(id) => {
                self.v2p
                    .get(&v)
                    .copied()
                    .unwrap_or(x86Val::mem(id + 16, 0, ty.size()))
            }
            LirValKind::Mem(reg) => self
                .v2p
                .get(&v)
                .copied()
                .expect("Could not find pointer in v2p map"),
            // .unwrap_or(x86Val::mem(reg, 0, v.size)),
            LirValKind::Imm(_) => panic!("Resolve pointer called on immediate value"),
        };
        assert!(matches!(ret.kind, Mem(..)));
        ret
    }

    fn get_val(&self, ty: &RealType, v: LirVal, builder: &mut Builder<x86Instr>) -> x86Val {
        let ret = match v.kind {
            LirValKind::Reg(id) => self
                .v2p
                .get(&v)
                .copied()
                .unwrap_or(x86Val::reg(id + 16, v.size)),
            LirValKind::Mem(..) => {
                let ptr = self.get_mem(ty, v);
                let reg = x86Val::reg(builder.new_reg(), 8);
                builder.emit(Lea(reg, ptr));
                reg
            }
            LirValKind::Imm(i) => x86Val::imm(i, v.size),
        };
        ret
    }

    fn sub_rsp(&mut self, amount: i128, builder: &mut Builder<x86Instr>) {
        self.v_rsp -= amount;
        builder.emit(Sub(RSP, x86Val::imm(amount as i128, 8)));
    }

    pub fn translate_func(&mut self, f: Builder<LirInstr>) -> Builder<x86Instr> {
        let mut builder = Builder::new(f.name, f.bb_count, f.vreg_count + 16);

        let prologue = builder.new_bb("prologue");
        builder.start_new_block(prologue);
        builder.emit(Push(RBP));
        builder.emit(Mov(RBP, RSP));

        let epilogue = builder.new_bb("epilogue");

        let mut bb_iter = f.bbs.iter().peekable();
        while let Some(bb) = bb_iter.next() {
            builder.start_new_block(bb.name);
            for i in bb.instructions.iter() {
                match i.clone() {
                    LirInstr::Comment(s) => {
                        if CFG.verbose {
                            builder.emit(Comment(s));
                        }
                    }
                    // Stack arg. This IR instruction just ensures that the num'th argument is moved
                    // to the stack
                    LirInstr::Stkarg(ty, dst, name, num) => {
                        let mut loc = x86Val::sysv_arg_n(num, ty.lookup().size());
                        if num <= 5 {
                            // This means loc is a register. We must move it to the stack
                            let stk_loc = x86Val::mem(BP, self.v_rsp - 8, ty.lookup().size());
                            let aligned_size = align_n(ty.lookup().size() as i128, 16);
                            self.sub_rsp(aligned_size, &mut builder);
                            builder.emit(Mov(stk_loc, loc));
                            loc = stk_loc;
                        }
                        builder.emit(Comment(format!("{name} ({ty}): {dst} -> {loc}")));
                        self.v2p.insert(dst, loc);
                    }
                    LirInstr::Arg(ty, dst, name, num) => {
                        let loc = x86Val::sysv_arg_n(num, ty.lookup().size());
                        builder.emit(Comment(format!("{name} ({ty}): {dst} -> {loc}")));
                        self.v2p.insert(dst, loc);
                    }
                    LirInstr::Alloc(ty, dst, name) => {
                        let loc = x86Val::mem(BP, self.v_rsp - 8, ty.lookup().size());
                        // TODO: alignment correction should be done after an "sroa" pass i think
                        let aligned_size = align_n(ty.lookup().size() as i128, 16);
                        self.sub_rsp(aligned_size, &mut builder);
                        builder.emit(Comment(format!("{name} ({ty}): {dst} -> {loc}")));
                        self.v2p.insert(dst, loc);
                    }
                    LirInstr::Copy(ty, dst, rs1) => {
                        let dst = self.get_val(ty.lookup(), dst, &mut builder);
                        let rs1 = self.get_val(ty.lookup(), rs1, &mut builder);
                        builder.emit(Mov(dst, rs1))
                    }
                    // mov ..., [rs1]
                    LirInstr::Load(ty, dst, rs1) => {
                        let dst = self.get_val(ty.lookup(), dst, &mut builder);
                        let rs1 = self.get_mem(ty.lookup(), rs1);

                        builder.emit(Mov(dst, rs1));
                    }
                    // mov [rs1], ...
                    LirInstr::Store(ty, rs1, rs2) => {
                        let rs1 = self.get_mem(ty.lookup(), rs1);
                        let rs2 = self.get_val(ty.lookup(), rs2, &mut builder);

                        builder.emit(Mov(rs1, rs2));
                    }
                    LirInstr::Br(ty, rs1, bb1, bb2) => {
                        let rs1 = self.get_val(ty.lookup(), rs1, &mut builder);
                        builder.emit(Cmp(rs1, x86Val::imm(1, ty.lookup().size())));
                        // If we fall through to the "then" block, there's no need to emit a `jnz`
                        if bb_iter.peek().is_none_or(|next_bb| next_bb.name != bb1) {
                            builder.emit(Jnz(bb1));
                        }
                        builder.emit(Jz(bb2));
                    }
                    LirInstr::Jmp(bb) => builder.emit(Jmp(bb)),
                    LirInstr::Add(ty, dst, rs1, rs2) => {
                        let dst = self.get_val(ty.lookup(), dst, &mut builder);
                        let rs1 = self.get_val(ty.lookup(), rs1, &mut builder);
                        let rs2 = self.get_val(ty.lookup(), rs2, &mut builder);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(Add(dst, rs2));
                    }
                    LirInstr::Sub(ty, dst, rs1, rs2) => {
                        let dst = self.get_val(ty.lookup(), dst, &mut builder);
                        let rs1 = self.get_val(ty.lookup(), rs1, &mut builder);
                        let rs2 = self.get_val(ty.lookup(), rs2, &mut builder);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(Sub(dst, rs2));
                    }
                    LirInstr::Smul(ty, dst, rs1, rs2) => {
                        let dst = self.get_val(ty.lookup(), dst, &mut builder);
                        let rs1 = self.get_val(ty.lookup(), rs1, &mut builder);
                        let rs2 = self.get_val(ty.lookup(), rs2, &mut builder);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(Imul(dst, rs2));
                    }
                    LirInstr::Umul(ty, dst, rs1, rs2) => {
                        // let dst = RAX;
                        // let rs1 = self.resolve_val(ty.lookup(), rs1, &mut builder);
                        // let rs2 = self.resolve_reg(rs2, &mut builder);
                        // builder.emit(Mov(dst, rs1));
                        // builder.emit(Mul(dst, rs2));
                        todo!("Implement unsigned multiply (mul)");
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
                        let rs1 = self.get_val(ty.lookup(), rs1, &mut builder);
                        let rs2 = self.get_val(ty.lookup(), rs2, &mut builder);
                        let ty = RealType::I32;
                        let mut dst = self.get_val(&ty, dst, &mut builder);
                        dst.size = 4;
                        let mut tmp = self.get_val(
                            &ty,
                            LirVal::reg(builder.new_reg(), ty.size()),
                            &mut builder,
                        );
                        tmp.size = 4;
                        builder.emit(Mov(tmp, x86Val::imm(0, ty.size())));
                        builder.emit(Cmp(rs1, rs2));
                        builder.emit(Mov(dst, x86Val::imm(1, ty.size())));
                        let cmov_instr = match i {
                            LirInstr::Eq(ty, ..) => Cmove(dst, tmp),
                            LirInstr::Sgt(ty, ..) => Cmovg(dst, tmp),
                            LirInstr::Sge(ty, ..) => Cmovge(dst, tmp),
                            LirInstr::Slt(ty, ..) => Cmovl(dst, tmp),
                            LirInstr::Sle(ty, ..) => Cmovle(dst, tmp),
                            LirInstr::Ugt(ty, ..) => Cmovg(dst, tmp),
                            LirInstr::Uge(ty, ..) => Cmovge(dst, tmp),
                            LirInstr::Ult(ty, ..) => Cmovl(dst, tmp),
                            LirInstr::Ule(ty, ..) => Cmovle(dst, tmp),
                            _ => unreachable!(),
                        };
                        builder.emit(cmov_instr);
                    }

                    LirInstr::Ret(ty, rs1) => {
                        let rs1 = self.get_val(ty.lookup(), rs1, &mut builder);
                        builder.emit(Mov(x86Val::reg(A, ty.lookup().size()), rs1));
                        builder.emit(Jmp(epilogue));
                    }

                    LirInstr::Retv => {
                        builder.emit(Jmp(epilogue));
                    }
                    LirInstr::Udiv(ty, lir_val, lir_val1, lir_val2) => todo!(),
                    LirInstr::Sdiv(ty, lir_val, lir_val1, lir_val2) => todo!(),
                    LirInstr::Trunc(ty, rd, rs1, rs2) => todo!(),
                    LirInstr::Zext(ty, rd, rs1, rs2) => {}
                    LirInstr::Sext(ty, rd, rs1, rs2) => todo!(),
                }
            }
        }
        builder.start_new_block(epilogue);
        builder.emit(Mov(RSP, RBP));
        builder.emit(Pop(RBP));
        builder.emit(Ret);

        let exit_bb = builder.new_bb("");
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
                        Reg(reg) => {
                            def[bbid].insert(reg);
                        }
                        Mem(reg, _) => {
                            if !def[bbid].contains(reg) {
                                use_[bbid].insert(reg);
                            }
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

        let mut graph = VecVecGraph::new(total_regs);

        for bb in builder.bbs.iter_mut() {
            let index = map_index[&bb.name];
            let mut live = &mut live_out[index];

            for i in bb.instructions.iter_mut().rev() {
                for dst in i.dsts() {
                    match dst.kind {
                        Reg(reg) => {
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
                        // Only dsts which are true registers are defs
                        Reg(reg) => {
                            live.remove(reg);
                        }
                        // [reg] being a dst still means reg is a use, not a def
                        Mem(reg, _) => {
                            live.insert(reg);
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
        // if CFG.verbose {
        //     for (v, es) in graph.iter().enumerate() {
        //         eprintln!("{v}: {es:?}");
        //     }
        // }

        if CFG.no_regalloc {
            return;
        }

        for bb in builder.bbs.iter_mut() {
            for i in bb.instructions.iter_mut() {
                for dst in i.dsts() {
                    match &mut dst.kind {
                        Reg(reg) | Mem(reg, _) => {
                            // TODO: This is NOT how precoloring works. Fix it!!
                            // True precoloring requires integration with the graph coloring
                            // algorithm itself, meaning I either need to find a better crate to do
                            // it for me, or fork it myself (likely).
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
                            // TODO: This is NOT how precoloring works. Fix it!!
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
