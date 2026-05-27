use std::{collections::HashMap, hash::Hash, ops::Deref, sync::LazyLock};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum TirType {
    // Primitive types
    I32,
    U32,
    Bool,
    Void,
    // Rest
    Base(&'static str),
    Function { args: Vec<Id>, returns: Id },
    Pointer(Id),
}

impl TirType {
    pub fn is_integral(&self) -> bool {
        matches!(self, TirType::I32 | TirType::U32)
    }

    pub fn is_signed(&self) -> bool {
        matches!(self, TirType::I32)
    }

    pub fn id(self) -> Id {
        unsafe { TYPES.add(self) }
    }
}

impl std::fmt::Display for TirType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TirType::I32 => format_args!("i32"),
            TirType::U32 => format_args!("u32"),
            TirType::Bool => format_args!("bool"),
            TirType::Void => format_args!("void"),
            TirType::Base(s) => format_args!("{}", *s),
            TirType::Function { args, returns } => {
                format_args!(
                    "Fn({}) -> {}",
                    args.iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    *returns
                )
            }
            TirType::Pointer(type_id) => format_args!("*{}", *type_id),
        };
        f.write_fmt(s)
    }
}

pub struct Store<K> {
    map: HashMap<K, Id>,
    vec: Vec<K>,
}

impl<K: Hash + Eq + Clone> Store<K> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, value: K) -> Id {
        if let Some(id) = self.map.get(&value) {
            return *id;
        }
        let id = Id(self.vec.len());
        self.vec.push(value.clone());
        self.map.insert(value, id);
        id
    }

    pub fn get(&self, value: &K) -> Option<Id> {
        self.map.get(value).copied()
    }

    pub fn lookup(&mut self, id: Id) -> &K {
        &self.vec[*id]
    }
}

impl<K> Default for Store<K> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            vec: Default::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(usize);

impl Deref for Id {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn lookup_type(id: Id) -> &'static TirType {
    unsafe { TYPES.lookup(id) }
}

pub fn get_type(value: &TirType) -> Option<Id> {
    unsafe { TYPES.get(value) }
}

pub fn add_type(value: TirType) -> Id {
    unsafe { TYPES.add(value) }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &TirType = lookup_type(*self);
        f.write_fmt(format_args!("{s}"))
    }
}

pub static mut TYPES: LazyLock<Store<TirType>> = LazyLock::new(|| {
    let mut s = Store::new();
    s.add(TirType::I32);
    s.add(TirType::U32);
    s.add(TirType::Bool);
    s.add(TirType::Void);
    s
});
