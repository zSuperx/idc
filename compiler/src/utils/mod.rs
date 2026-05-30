use std::hash::Hash;
use std::{collections::HashMap, ops::Add};

pub fn align_n(x: i128, n: usize) -> i128 {
    (x + n as i128) & !n as i128
}

#[derive(Debug)]
pub struct Env<K, V> {
    scopes: Vec<HashMap<K, V>>,
}

impl<K, V> Default for Env<K, V> {
    fn default() -> Self {
        Self {
            scopes: Default::default(),
        }
    }
}

impl<K: Hash + Eq, V: Copy> Env<K, V> {
    pub fn insert(&mut self, name: K, val: V) -> Option<V> {
        self.scopes.last_mut().unwrap().insert(name, val)
    }

    pub fn get(&self, name: &K) -> Option<V> {
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
