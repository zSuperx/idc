use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::rc::Rc;

use registry::{Id, Registry};

use crate::stir::isa::IRType;

use super::basicblock::{BBID, BasicBlock};
use super::traits::InstructionTrait;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructDef {
    pub(crate) size: usize,
    pub(crate) alignment: usize,
    pub(crate) fields: Vec<IRType>,
}

pub type StructId = Id<StructDef>;

#[derive(Debug, Clone)]
pub struct FunctionBuilder<I: InstructionTrait, V, T> {
    pub(crate) name: &'static str,
    pub(crate) cursor: BBID<I>,
    pub(crate) args: Vec<(V, T)>,
    pub(crate) return_type: T,
    pub(crate) entrypoint: BBID<I>,
    pub(crate) reg_count: usize,
    pub(crate) block_count: usize,
    pub(crate) structs: Rc<RefCell<Registry<StructDef>>>,
    pub(crate) blocks: HashMap<BBID<I>, BasicBlock<I>>,
}

/// Given an `IRBuilder<I>` and format args, expands to `$builder.emit(Comment(format!(...)))`
///
/// This relies on the existence of an `I::Comment(String)` instruction for the instruction type of
/// the `IRBuilder`.
#[macro_export]
macro_rules! comment {
    ($builder:expr, $($fmtargs:tt)*) => {
        $builder.emit(Comment(format!($($fmtargs)*)))
    }
}

impl<I: InstructionTrait, V, T> FunctionBuilder<I, V, T> {
    /// Creates a new IR Function builder. The function is initialized with an empty BasicBlock as
    /// its entrypoint.
    ///
    /// The insert point is set to this entrypoint, so you can start emitting immediately after
    /// creating it.
    pub fn new(name: &'static str, return_type: T) -> Self {
        let cursor = BBID(name, "entrypoint", 0, PhantomData::default());
        let blocks = HashMap::from([(cursor, BasicBlock::empty())]);
        let block_count = 1;
        let reg_count = 0;

        Self {
            cursor,
            name,
            return_type,
            reg_count,
            block_count,
            blocks,
            entrypoint: cursor,
            args: Default::default(),
            structs: Default::default(),
        }
    }

    pub fn addArg(&mut self, arg_value: V, arg_type: T) {
        self.args.push((arg_value, arg_type));
    }

    pub fn addStruct(&mut self, s: StructDef) -> Id<StructDef> {
        self.structs.borrow_mut().add(s)
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
        F: FnMut(BBID<I>, &BasicBlock<I>) -> bool,
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
        F: FnMut(BBID<I>, &mut BasicBlock<I>) -> bool,
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

    /// Performs a visitor pass that verifies no reachable block in the function lacks a terminator.
    ///
    /// Upon finding an invalid block, the verifier either breaks with `false`, OR if a
    /// `default_return` instruction is provided, will set the block's terminator to that.
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

    pub fn setInsertPoint(&mut self, block: BBID<I>) {
        self.cursor = block;
    }

    pub fn getInsertPoint(&mut self) -> BBID<I> {
        self.cursor
    }

    pub fn getReturnType(&self) -> &T {
        &self.return_type
    }

    pub fn newBlock(&mut self) -> BBID<I> {
        let id = BBID(self.name, "", self.block_count, Default::default());
        self.block_count += 1;
        self.blocks.insert(id, BasicBlock::empty());
        id
    }

    pub fn newNamedBlock(&mut self, block_name: &'static str) -> BBID<I> {
        let id = BBID(self.name, block_name, self.block_count, Default::default());
        self.block_count += 1;
        self.blocks.insert(id, BasicBlock::new(block_name));
        id
    }

    /// Retrieves the entrypoint of the function
    pub fn getEntryPoint(&self) -> BBID<I> {
        self.entrypoint
    }

    /// Replaces a function's entrypoint, returning the old one
    pub fn setEntryPoint(&mut self, new_entrypoint: BBID<I>) -> BBID<I> {
        let ret = self.entrypoint;
        self.entrypoint = new_entrypoint;
        ret
    }

    #[inline]
    pub fn removeFallthrough(&mut self) -> Option<BBID<I>> {
        self.removeFallthroughFrom(self.cursor)
    }

    pub fn removeFallthroughFrom(&mut self, this: BBID<I>) -> Option<BBID<I>> {
        let this_block = self.blocks.get_mut(&this).unwrap();
        this_block.fallthrough.take()
    }

    #[inline]
    pub fn addFallthrough(&mut self, fallthrough: BBID<I>) {
        self.addFallthroughTo(self.cursor, fallthrough);
    }

    pub fn addFallthroughTo(&mut self, this: BBID<I>, fallthrough: BBID<I>) {
        let this_block = self.blocks.get_mut(&this).unwrap();
        this_block.fallthrough = Some(fallthrough);
        this_block.successors.insert(fallthrough);

        let other_block = self.blocks.get_mut(&fallthrough).unwrap();
        other_block.predecessors.insert(this);
    }

    #[inline]
    pub fn addSuccessors(&mut self, successors: &[BBID<I>]) {
        self.addSuccessorsTo(self.cursor, successors);
    }

    pub fn addSuccessorsTo(&mut self, this: BBID<I>, successors: &[BBID<I>]) {
        for succ_id in successors {
            let this_block = self.blocks.get_mut(&this).unwrap();
            this_block.successors.insert(*succ_id);

            let other_block = self.blocks.get_mut(succ_id).unwrap();
            other_block.predecessors.insert(this);
        }
    }

    #[inline]
    pub fn addPredecessors(&mut self, successors: &[BBID<I>]) {
        self.addPredecessorsTo(self.cursor, successors);
    }

    pub fn addPredecessorsTo(&mut self, this: BBID<I>, predecessors: &[BBID<I>]) {
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

    pub fn isTerminated(&self, this: BBID<I>) -> bool {
        self.blocks[&this].terminator.is_some()
    }

    pub fn nextReg(&mut self) -> usize {
        let ret = self.reg_count;
        self.reg_count += 1;
        ret
    }
}
