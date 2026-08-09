use crate::common::*;

#[derive(Debug)]
pub struct Builder<I> {
    pub name: &'static str,
    pub vreg_count: usize,
    // BBs we have built so far
    pub bbs: Vec<BasicBlock<I>>,
    pub bb_count: usize,
    // In-progress stuff for building BBs
    pub buf: Vec<I>,
    pub curr_bb_name: Option<BB>,
}

impl<I: std::fmt::Display> std::fmt::Display for Builder<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:\n", self.name))?;
        for bb in self.bbs.iter() {
            f.write_fmt(format_args!("{bb}"))?;
        }
        Ok(())
    }
}

impl<I> Default for Builder<I> {
    fn default() -> Self {
        Self {
            name: Default::default(),
            vreg_count: Default::default(),
            bbs: Default::default(),
            bb_count: Default::default(),
            buf: Default::default(),
            curr_bb_name: Default::default(),
        }
    }
}

impl<I: Clone + Instr> Builder<I> {
    pub fn new(name: &'static str, bb_count: usize, vreg_count: usize) -> Self {
        let mut s = Self::default();
        s.name = name;
        s.vreg_count = vreg_count;
        s.bb_count = bb_count;
        s
    }

    pub fn new_reg(&mut self) -> usize {
        let id = self.vreg_count;
        self.vreg_count += 1;
        id
    }

    pub fn new_bb(&mut self, name: &'static str) -> BB {
        self.bb_count += 1;
        BB(self.bb_count - 1, name)
    }

    pub fn start_new_block(&mut self, name: BB) {
        if let Some(old_name) = self.curr_bb_name {
            // Commit the old block, but first check if it terminated
            let tmp = self.get_terminator();
            let mut buf = std::mem::take(&mut self.buf);
            let terminator = match tmp {
                Some(t) => t,
                None => {
                    let chain = I::uncond_jump(name);
                    buf.push(chain.clone());
                    chain
                } // if it didn't terminate, hook it up to the new one
            };
            let bb = BasicBlock::new(old_name, buf, terminator);
            self.bbs.push(bb);
        }
        self.curr_bb_name = Some(name);
    }

    pub fn get_terminator(&self) -> Option<I> {
        self.buf.last().filter(|&i| i.is_terminator()).cloned()
    }

    pub fn emit(&mut self, instr: I) {
        if self.get_terminator().is_none() {
            self.buf.push(instr);
        }
    }
}
