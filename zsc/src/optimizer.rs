use std::collections::{HashMap, HashSet};

use crate::arch::lir::LirVal;
use crate::aux::Compiler;

use crate::prelude::*;

use LirInstr::*;

impl Compiler {
    pub fn optim_func(&mut self, mut builder: Builder<LirInstr>) -> Builder<LirInstr> {
        let mut const_map = HashMap::new();
        let mut is_read = HashSet::new();
        for mut bb in builder.bbs.iter_mut() {
            self.const_fold_bb(&mut bb, &mut const_map);
        }

        for mut bb in builder.bbs.iter_mut() {
            self.track_live_code(&mut bb, &mut is_read);
        }

        for mut bb in builder.bbs.iter_mut() {
            self.dead_code_elim(&mut bb, &mut is_read);
        }
        builder
    }

    fn const_fold_bb(&mut self, bb: &mut BasicBlock<LirInstr>, map: &mut HashMap<LirVal, i128>) {
        for i in 0..bb.instructions.len() {
            match &bb.instructions[i] {
                Copy(dst, rs1) => {
                    let v1 = map.get(&rs1).copied();
                    if let Some(imm) = v1 {
                        map.insert(*dst, imm);
                    }
                }
                op @ (Add(dst, rs1, rs2)
                | Sub(dst, rs1, rs2)
                | Smul(dst, rs1, rs2)
                | Umul(dst, rs1, rs2)
                | Sgt(dst, rs1, rs2)
                | Sge(dst, rs1, rs2)
                | Slt(dst, rs1, rs2)
                | Sle(dst, rs1, rs2)
                | Ugt(dst, rs1, rs2)
                | Uge(dst, rs1, rs2)
                | Ult(dst, rs1, rs2)
                | Ule(dst, rs1, rs2)
                | Eq(dst, rs1, rs2)) => {
                    let v1 = map.get(&rs1).copied();
                    let v2 = map.get(&rs2).copied();

                    match (v1, v2) {
                        (Some(imm1), Some(imm2)) => {
                            let uimm1 = imm1 as u128;
                            let uimm2 = imm2 as u128;
                            let imm = match op {
                                Add(..) => imm1 + imm2,
                                Sub(..) => imm1 - imm2,
                                Smul(..) => imm1 * imm2,
                                Umul(..) => (uimm1 * uimm2) as i128,
                                Eq(..) => (imm1 == imm2) as i128,
                                Sgt(..) => (imm1 > imm2) as i128,
                                Sge(..) => (imm1 >= imm2) as i128,
                                Slt(..) => (imm1 < imm2) as i128,
                                Sle(..) => (imm1 <= imm2) as i128,
                                Ugt(..) => (uimm1 > uimm2) as i128,
                                Uge(..) => (uimm1 >= uimm2) as i128,
                                Ult(..) => (uimm1 < uimm2) as i128,
                                Ule(..) => (uimm1 <= uimm2) as i128,
                                _ => unreachable!(),
                            };
                            map.insert(*dst, imm);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    /// This pass should simply mark which registers are read from
    fn track_live_code(&mut self, bb: &BasicBlock<LirInstr>, is_read: &mut HashSet<LirVal>) {
        for instr in bb.instructions.iter() {
            match *instr {
                Ret(rs1) | Br(rs1, ..) | Copy(_, rs1) | Load(_, rs1) => {
                    is_read.insert(rs1);
                }
                Add(_, rs1, rs2)
                | Sub(_, rs1, rs2)
                | Smul(_, rs1, rs2)
                | Umul(_, rs1, rs2)
                | Eq(_, rs1, rs2)
                | Sgt(_, rs1, rs2)
                | Sge(_, rs1, rs2)
                | Slt(_, rs1, rs2)
                | Sle(_, rs1, rs2)
                | Ugt(_, rs1, rs2)
                | Uge(_, rs1, rs2)
                | Ult(_, rs1, rs2)
                | Ule(_, rs1, rs2)
                | Store(rs1, rs2) => {
                    is_read.insert(rs1);
                    is_read.insert(rs2);
                }
                _ => {}
            }
        }
    }

    // This pass should look at all instructions who PRODUCE a value. If that value is read, it is
    // considered a useful instructions
    fn dead_code_elim(&mut self, bb: &mut BasicBlock<LirInstr>, is_read: &HashSet<LirVal>) {
        let old = std::mem::take(&mut bb.instructions);
        for instr in old {
            match instr {
                Copy(_, dst, ..)
                | Add(_, dst, ..)
                | Sub(_, dst, ..)
                | Smul(_, dst, ..)
                | Umul(_, dst, ..)
                | Eq(_, dst, ..)
                | Sgt(_, dst, ..)
                | Sge(_, dst, ..)
                | Slt(_, dst, ..)
                | Sle(_, dst, ..)
                | Ugt(_, dst, ..)
                | Uge(_, dst, ..)
                | Ult(_, dst, ..)
                | Ule(_, dst, ..)
                | Load(_, dst, ..) => {
                    if is_read.contains(&dst) {
                        bb.instructions.push(instr);
                    }
                }
                _ => bb.instructions.push(instr),
            }
        }
    }
}
