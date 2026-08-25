use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct Env<K: Hash + Eq, V: Copy> {
    scopes: Vec<HashMap<K, V>>,
}

impl<K: Hash + Eq, V: Copy> Default for Env<K, V> {
    fn default() -> Self {
        Self {
            scopes: vec![HashMap::default()],
        }
    }
}

impl<K: Hash + Eq, V: Copy> Env<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_first(&mut self, name: K, val: V) -> Option<V> {
        self.scopes.first_mut().unwrap().insert(name, val)
    }

    pub fn insert(&mut self, name: K, val: V) -> Option<V> {
        self.scopes.last_mut().unwrap().insert(name, val)
    }

    pub fn get_first(&self, name: &K) -> Option<V> {
        self.scopes.first().unwrap().get(name).copied()
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
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
}
