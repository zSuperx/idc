use std::collections::HashMap;

use crate::{ast::*, lir, tir::{Id, Store}};

#[derive(Default)]
pub struct Compiler {
    // Lexer shit
    pub cursor: usize,
    pub row: usize,
    pub col: usize,

    // Parser shit
    pub last_span: Span,

    // Type checker shit
    pub type_env: Env<Id>,
    pub var_store: Store<&'static str>,
    pub return_type: Id,
    pub current_fn_lvars: Vec<(&'static str, Id)>,

    // Lowering shit
    pub reg_count: usize,
    pub lbl_count: usize,
    pub bb_count: usize,
    pub bbs: Vec<lir::BasicBlock>,
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Default)]
pub struct Env<T> {
    scopes: Vec<HashMap<&'static str, T>>,
}

impl<T: Copy> Env<T> {
    pub fn insert(&mut self, name: &'static str, val: T) -> Option<T> {
        self.scopes.last_mut().unwrap().insert(name, val)
    }

    pub fn get(&self, name: &str) -> Option<T> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(*t);
            }
        }
        None
    }
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}
