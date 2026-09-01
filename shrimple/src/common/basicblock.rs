use std::{collections::HashSet, marker::PhantomData};

use super::traits::InstructionTrait;

#[derive(Debug, Default)]
pub struct BBID<I>(
    pub(crate) &'static str,
    pub(crate) &'static str,
    pub(crate) usize,
    pub(crate) PhantomData<I>,
);

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
    pub successors: HashSet<BBID<I>>,
    pub predecessors: HashSet<BBID<I>>,
    pub fallthrough: Option<BBID<I>>,
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

    pub fn rewrite_instructions(&mut self, mut rewriter: impl FnMut(&I) -> RewriteAction<I>) {
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
