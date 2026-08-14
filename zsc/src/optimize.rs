use std::collections::{HashMap, HashSet};

use crate::arch::lir::*;
use crate::aux::Compiler;

use crate::common::*;

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

    fn const_fold_bb(&mut self, bb: &mut BasicBlockOld<LirInstr>, map: &mut HashMap<Value, i128>) {
        for i in 0..bb.instructions.len() {
            match &bb.instructions[i] {
                Copy(_, dst, rs1) => {
                    let v1 = map.get(&rs1).copied();
                    if let Some(imm) = v1 {
                        map.insert(*dst, imm);
                    }
                }
                op @ (Trunc(ty, dst, rs1) | Zext(ty, dst, rs1) | Sext(ty, dst, rs1)) => {
                    let v1 = map.get(&rs1).copied();
                    if let Some(imm) = v1 {
                        map.insert(*dst, imm);
                    }
                    if let ValueKind::Imm(imm) = rs1.kind {
                        map.insert(*dst, imm);
                    }
                }
                op @ (Add(_, dst, rs1, rs2)
                | Sub(_, dst, rs1, rs2)
                | Smul(_, dst, rs1, rs2)
                | Umul(_, dst, rs1, rs2)
                | Sgt(_, dst, rs1, rs2)
                | Sge(_, dst, rs1, rs2)
                | Slt(_, dst, rs1, rs2)
                | Sle(_, dst, rs1, rs2)
                | Ugt(_, dst, rs1, rs2)
                | Uge(_, dst, rs1, rs2)
                | Ult(_, dst, rs1, rs2)
                | Ule(_, dst, rs1, rs2)
                | Eq(_, dst, rs1, rs2)) => {
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
    fn track_live_code(&mut self, bb: &mut BasicBlockOld<LirInstr>, is_read: &mut HashSet<Value>) {
        for instr in bb.instructions.iter_mut() {
            for src in instr.srcs() {
                is_read.insert(*src);
            }
        }
    }

    // This pass should look at all instructions who PRODUCE a value. If that value is read, it is
    // considered a useful instructions, otherwise it's dead code
    fn dead_code_elim(&mut self, bb: &mut BasicBlockOld<LirInstr>, is_read: &HashSet<Value>) {
        let old = std::mem::take(&mut bb.instructions);
        for mut instr in old {
            let mut keep = false;
            if instr.is_terminator() || matches!(instr, Store(..) | Comment(..)) {
                keep = true;
            } else {
                for dst in instr.dsts() {
                    if is_read.contains(&dst) {
                        keep = true;
                        break;
                    }
                }
            }

            if keep {
                bb.instructions.push(instr);
            }
        }
    }
}
