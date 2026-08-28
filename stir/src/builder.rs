use std::collections::{HashMap, HashSet};

use crate::traits::InstructionTrait;

#[derive(Debug, Clone, Copy, Default, Hash, Eq, PartialEq)]
pub struct BBID(&'static str, &'static str, usize);

#[derive(Debug, Clone)]
pub struct IRFunction<I: InstructionTrait, T> {
    cursor: BBID,
    name: &'static str,
    return_type: T,
    entrypoint: BBID,
    reg_count: usize,
    block_count: usize,
    blocks: HashMap<BBID, BasicBlock<I>>,
}

#[macro_export]
/// Given an `IRBuilder<I>` and format args, expands to `$builder.emit(Comment(format!(...)))`
///
/// This relies on the existence of an `I::Comment(String)` instruction for the instruction type of
/// the `IRBuilder`.
macro_rules! comment {
    ($builder:expr, $($fmtargs:tt)*) => {
        $builder.emit(Comment(format!($($fmtargs)*)))
    }
}

impl std::fmt::Display for BBID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let BBID(function, name, index) = self;
        if name.is_empty() {
            f.write_fmt(format_args!(".{function}.{index}"))
        } else {
            f.write_fmt(format_args!(".{name}.{index}"))
        }
    }
}

#[derive(Debug, Clone)]
pub struct BasicBlock<I: InstructionTrait> {
    pub name: &'static str,
    pub successors: HashSet<BBID>,
    pub predecessors: HashSet<BBID>,
    pub fallthrough: Option<BBID>,
    pub instructions: Vec<I>,
    pub terminator: Option<I>,
}

impl<I: InstructionTrait> BasicBlock<I> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            successors: Default::default(),
            predecessors: Default::default(),
            fallthrough: Default::default(),
            instructions: Default::default(),
            terminator: Default::default(),
        }
    }

    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            successors: Default::default(),
            predecessors: Default::default(),
            fallthrough: Default::default(),
            instructions: Default::default(),
            terminator: Default::default(),
        }
    }
}

impl<I: InstructionTrait, T> IRFunction<I, T> {
    /// Creates a new IR Function builder. The function is initialized with an empty BasicBlock as
    /// its entrypoint.
    ///
    /// The insert point is set to this entrypoint, so you can start emitting immediately after
    /// creating it.
    pub fn new(name: &'static str, return_type: T) -> Self {
        let cursor = BBID(name, "entrypoint", 0);
        let blocks = HashMap::from([(cursor, BasicBlock::empty())]);
        let block_count = 1;
        let reg_count = 0;

        Self {
            cursor,
            name,
            return_type,
            entrypoint: cursor,
            reg_count,
            block_count,
            blocks,
        }
    }

    /// Returns the name of the function
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn getRegCount(&self) -> usize {
        self.reg_count
    }

    pub fn setRegCount(&mut self, new: usize) {
        self.reg_count = new;
    }

    /// Perform a DFS visitor pass through the function
    ///
    /// `visitor`: a function that accepts both the current block and its `id` and returns whether
    /// the DFS should end early.
    ///
    /// the `dfs()` function itself will return whether or not DFS ended early. That is, it will
    /// mimic `visitor`'s return value
    pub fn dfs<F>(&self, mut visitor: F) -> bool
    where
        F: FnMut(BBID, &BasicBlock<I>) -> bool,
    {
        let mut seen = HashSet::new();
        let mut stack = vec![self.entrypoint];
        while let Some(id) = stack.pop() {
            seen.insert(id);
            let block = self.blocks.get(&id).unwrap();

            if visitor(id, block) {
                return true;
            }

            for succ in block.successors.iter() {
                if !seen.contains(succ) && Some(*succ) != block.fallthrough {
                    stack.push(*succ);
                }
            }

            if let Some(ft) = block.fallthrough {
                if !seen.contains(&ft) {
                    stack.push(ft);
                }
            }
        }
        false
    }

    /// Perform a DFS mutable visitor pass through the function
    ///
    /// `visitor`: a function that accepts both the current block and its `id` and returns whether
    /// the DFS should end early.
    ///
    /// the `dfs()` function itself will return whether or not DFS ended early. That is, it will
    /// mimic `visitor`'s return value
    pub fn dfs_mut<F>(&mut self, mut visitor: F) -> bool
    where
        F: FnMut(BBID, &mut BasicBlock<I>) -> bool,
    {
        let mut seen = HashSet::new();
        let mut stack = vec![self.entrypoint];
        while let Some(id) = stack.pop() {
            seen.insert(id);
            let block = self.blocks.get_mut(&id).unwrap();

            if visitor(id, block) {
                return true;
            }

            for succ in block.successors.iter() {
                if !seen.contains(succ) && Some(*succ) != block.fallthrough {
                    stack.push(*succ);
                }
            }

            if let Some(ft) = block.fallthrough {
                if !seen.contains(&ft) {
                    stack.push(ft);
                }
            }
        }
        false
    }

    pub fn verify(&mut self, default_return: Option<I>) -> bool {
        !self.dfs_mut(|_id, block| {
            if block.terminator.is_none() {
                match default_return.as_ref() {
                    Some(i) => block.terminator = Some(i.clone()),
                    None => return true,
                }
            }
            false
        })
    }

    pub fn print(&self) {
        println!("{}:", self.name);
        self.dfs(|id, block| {
            println!("{id}:");
            for i in block.instructions.iter() {
                println!("\t{i}");
            }
            if let Some(term) = &block.terminator {
                println!("\t{term}");
            } else {
                println!("\t; !! (missing terminator)");
            }
            false
        });
    }

    pub fn setInsertPoint(&mut self, block: BBID) {
        self.cursor = block;
    }

    pub fn getInsertPoint(&mut self) -> BBID {
        self.cursor
    }

    pub fn getReturnType(&self) -> &T {
        &self.return_type
    }

    pub fn newBlock(&mut self) -> BBID {
        let id = BBID(self.name, "", self.block_count);
        self.block_count += 1;
        self.blocks.insert(id, BasicBlock::empty());
        id
    }

    pub fn newNamedBlock(&mut self, block_name: &'static str) -> BBID {
        let id = BBID(self.name, block_name, self.block_count);
        self.block_count += 1;
        self.blocks.insert(id, BasicBlock::new(block_name));
        id
    }

    /// Retrieves the entrypoint of the function
    pub fn getEntryPoint(&self) -> BBID {
        self.entrypoint
    }

    /// Replaces a function's entrypoint, returning the old one
    pub fn setEntryPoint(&mut self, new_entrypoint: BBID) -> BBID {
        let ret = self.entrypoint;
        self.entrypoint = new_entrypoint;
        ret
    }

    #[inline]
    pub fn removeFallthrough(&mut self) -> Option<BBID> {
        self.removeFallthroughFrom(self.cursor)
    }

    pub fn removeFallthroughFrom(&mut self, this: BBID) -> Option<BBID> {
        let this_block = self.blocks.get_mut(&this).unwrap();
        this_block.fallthrough.take()
    }

    #[inline]
    pub fn addFallthrough(&mut self, fallthrough: BBID) {
        self.addFallthroughTo(self.cursor, fallthrough);
    }

    pub fn addFallthroughTo(&mut self, this: BBID, fallthrough: BBID) {
        let this_block = self.blocks.get_mut(&this).unwrap();
        this_block.fallthrough = Some(fallthrough);
        this_block.successors.insert(fallthrough);

        let other_block = self.blocks.get_mut(&fallthrough).unwrap();
        other_block.predecessors.insert(this);
    }

    #[inline]
    pub fn addSuccessors(&mut self, successors: &[BBID]) {
        self.addSuccessorsTo(self.cursor, successors);
    }

    pub fn addSuccessorsTo(&mut self, this: BBID, successors: &[BBID]) {
        for succ_id in successors {
            let this_block = self.blocks.get_mut(&this).unwrap();
            this_block.successors.insert(*succ_id);

            let other_block = self.blocks.get_mut(succ_id).unwrap();
            other_block.predecessors.insert(this);
        }
    }

    #[inline]
    pub fn addPredecessors(&mut self, successors: &[BBID]) {
        self.addPredecessorsTo(self.cursor, successors);
    }

    pub fn addPredecessorsTo(&mut self, this: BBID, predecessors: &[BBID]) {
        for pred_id in predecessors {
            assert_eq!(this.0, pred_id.0);
            let this_block = self.blocks.get_mut(&this).unwrap();
            this_block.predecessors.insert(*pred_id);

            let other_block = self.blocks.get_mut(pred_id).unwrap();
            other_block.successors.insert(this);
        }
    }

    pub fn emit(&mut self, instr: I) {
        let basic_block = self.blocks.get_mut(&self.cursor).unwrap();
        if basic_block.terminator.is_some() {
            return;
        }
        if instr.is_terminator() {
            basic_block.terminator = Some(instr);
        } else {
            basic_block.instructions.push(instr);
        }
    }

    pub fn isCurrentTerminated(&self) -> bool {
        self.blocks[&self.cursor].terminator.is_some()
    }

    pub fn isTerminated(&self, this: BBID) -> bool {
        self.blocks[&this].terminator.is_some()
    }

    pub fn nextReg(&mut self) -> usize {
        let ret = self.reg_count;
        self.reg_count += 1;
        ret
    }
}
