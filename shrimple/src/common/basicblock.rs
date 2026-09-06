//! This module defines the [`BasicBlock`] struct, which represents a bundle of instructions
//! terminated by a control-flow-inducing instruction.
//!
//! [`BBID`] is meant to be an index marker for `BasicBlock`s. Hence, [`FunctionBuilder`] and all
//! its implementations use `BBID` as a key into a map of `BasicBlock`s. 
use std::{collections::BTreeSet, marker::PhantomData};

use bitset::BitSet;

use super::traits::InstructionTrait;

#[derive(Debug, Default)]
pub struct BBID<I>(
    pub(crate) &'static str,
    pub(crate) &'static str,
    pub(crate) usize,
    pub(crate) PhantomData<I>,
);

impl<I> PartialOrd for BBID<I> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.0.partial_cmp(&other.0) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.1.partial_cmp(&other.1) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.2.partial_cmp(&other.2) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.3.partial_cmp(&other.3)
    }
}

impl<I> Ord for BBID<I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.2.cmp(&other.2)
    }
}

impl<I> Copy for BBID<I> {}
impl<I> Clone for BBID<I> {
    fn clone(&self) -> Self {
        Self(self.0, self.1, self.2, self.3)
    }
}

impl<I> std::hash::Hash for BBID<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
    }
}

impl<I> Eq for BBID<I> {}
impl<I> PartialEq for BBID<I> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

impl<I> std::fmt::Display for BBID<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let BBID(function, name, index, _) = self;
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
    pub successors: BTreeSet<BBID<I>>,
    pub predecessors: BTreeSet<BBID<I>>,
    pub fallthrough: Option<BBID<I>>,
    pub instructions: Vec<I>,
    pub terminator: Option<I>,

    pub live_in: BitSet,
    pub live_out: BitSet,
    pub gen_: BitSet,
    pub kill: BitSet,
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
            live_in: Default::default(),
            live_out: Default::default(),
            gen_: Default::default(),
            kill: Default::default(),
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
            live_in: Default::default(),
            live_out: Default::default(),
            gen_: Default::default(),
            kill: Default::default(),
        }
    }

    /// This function calls the provided `rewriter` closure on each instruction within the
    /// `BasicBlock`. The closure returns a [`RewriteAction`], which this function uses to determine
    /// how to proceed with the rewrite.
    pub fn rewrite(&mut self, mut rewriter: impl FnMut(&I) -> RewriteAction<I>) {
        let old = std::mem::take(&mut self.instructions);
        for i in old {
            match rewriter(&i) {
                RewriteAction::Skip => continue,
                RewriteAction::Keep => self.instructions.push(i),
                RewriteAction::Replace(items) => self.instructions.extend(items),
                RewriteAction::InsertBefore(items) => {
                    self.instructions.extend(items);
                    self.instructions.push(i);
                }
                RewriteAction::InsertAfter(items) => {
                    self.instructions.push(i);
                    self.instructions.extend(items);
                }
            }
        }
    }
}

pub enum RewriteAction<I: InstructionTrait> {
    Skip,
    Keep,
    Replace(Vec<I>),
    InsertBefore(Vec<I>),
    InsertAfter(Vec<I>),
}
