
use std::collections::HashMap;

use std::hash::Hash;
use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

use std::ops::Index;

use ::lexer::Token;

type Key = u64;

pub struct Symtab {
    hash_maps: HashMap<Key, SymtabNode>,
    current: Key,
    pub placeholder_symbol: bool,
    pub allow_update: bool,
}

struct SymtabNode {
    parent: Option<Key>,
    name: Option<(String, Token)>, // For DEBUG
    hash_map: HashMap<String, Symbol>,
    children: Vec<Key>,
}

impl SymtabNode {
    fn new(parent: Option<Key>, name: Option<(String, Token)>) -> SymtabNode {
        SymtabNode {
            parent: parent,
            name: name,
            hash_map: HashMap::new(),
            children: Vec::new(),
        }
    }
}

// Produce symtab populated with registers

use ::arch::inst::register::abi_name;

static placeholder_symbol: Symbol = Symbol {
    ext: false,
    mutable: false,
    value: Value::General(0),
};

#[derive(Debug)]
pub enum SymtabError {
    NoSymbol(String),
    DuplicateSymbol(String),
}

impl Symtab {
    pub fn new() -> Symtab {
        let root = SymtabNode::new(None, None);

        let mut hash_maps = HashMap::<Key, SymtabNode>::new();

        let key_root = Symtab::get_hash(&root.name);

        hash_maps.insert(key_root, root);

        Symtab {
            hash_maps: hash_maps,
            current: key_root,
            placeholder_symbol: false,
            allow_update: false,
        }
    }

    fn get_hash(name: &Option<(String, Token)>) -> u64 {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        hasher.finish()
    }

    fn current_node_mut<'a>(&'a mut self) -> &'a mut SymtabNode {
        let key = self.current;
        self.hash_maps.get_mut(&key).unwrap()
    }

    fn current_node<'a>(&'a self) -> &'a SymtabNode {
        let key = self.current;
        &self.hash_maps[&key]
    }

    pub fn prepopulated() -> Symtab {
        let mut symtab = Symtab::new();

        for i in 0u8..32u8 {
            let name = abi_name(i);

            let sym = Symbol {
                ext: false,
                mutable: false,
                value: Value::Register(i),
            };

            symtab.insert(name, sym).unwrap();
        }

        symtab
    }

    pub fn push_env(&mut self, name: (String, Token)) {
        // see if child exists
        let name = Some(name);
        let key = Symtab::get_hash(&name);

        if self.current_node().children.contains(&key) {
            if !self.allow_update {
                panic!("Duplicate child symtab node")
            }

            assert_eq!(self.hash_maps[&key].parent.unwrap(), self.current, "push_env consistency");

            // Push the existing
            self.current = key;

        } else {
            // Create new symtab node
            let node = SymtabNode::new(Some(self.current), name);
            self.current_node_mut().children.push(key);

            self.hash_maps.insert(key, node);

            self.current = key;
        }
    }

    pub fn pop_env(&mut self) {
        self.current = self.current_node().parent.unwrap();
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        match self.get_inner(self.current, name) {
            Some(s) => Some(s),
            None => {
                if self.placeholder_symbol {
                    Some(&placeholder_symbol)
                } else {
                    None
                }
            }
        }
    }

    // Get from current block, don't follow parents
    pub fn get_immediate(&self, name: &str) -> Option<&Symbol> {
        self.current_node().hash_map.get(name)
    }

    // search through parent nodes
    fn get_inner(&self, key: Key, name: &str) -> Option<&Symbol> {
        let node = &self.hash_maps[&key];

        match node.hash_map.get(name) {
            Some(v) => Some(v),
            None => {
                match node.parent {
                    Some(p) => self.get_inner(p, name),
                    None => None
                }
            }
        }
    }

    //TODO: maybe should handle shadowing?
    pub fn insert(&mut self, name: &str, sym: Symbol) -> Result<(), SymtabError> {
        if self.current_node_mut().hash_map.insert(String::from(name), sym).is_some() {
            if !self.allow_update {
                return Err(SymtabError::DuplicateSymbol(String::from(name)));
            }
        }

        Ok(())
    }

    pub fn update(&mut self, name: &str, sym: Symbol) -> Result<(), SymtabError> {
        if self.current_node_mut().hash_map.insert(String::from(name), sym).is_some() {
            Ok(())
        } else {
            Err(SymtabError::NoSymbol(String::from(name)))
        }
    }

    pub fn register_label(&mut self, name: &str, counter: u32) -> Result<(), SymtabError> {
        let sym = Symbol {
            ext: false,
            mutable: false,
            value: Value::Location(counter),
        };
        self.insert(name, sym)
    }

    // Debug
    pub fn print(&self) {
        println!("Symtab content:");
        for (k, v) in &self.current_node().hash_map {
            println!("{}: {:?}", k, &v.value);
        }
        println!();
    }
}

#[derive(Debug)]
pub struct Symbol {
    pub ext: bool,
    pub mutable: bool,
    pub value: Value,
}

impl Symbol {
    pub fn new(value: Value) -> Symbol {
        Symbol {
            ext: false,
            mutable: false,
            value: value,
        }
    }
}

#[derive(Debug)]
pub enum Value {
    General(i64),
    Location(u32),
    Register(u8),
    Csr(u32),

    // Internal use
    String(String),
    Strings(Vec<String>),
    Integers(Vec<i64>),
}

impl Value {
    pub fn eval_as_u32(&self) -> u32 {
        match self {
            &Value::General(v) => v as u32, // truncate...
            &Value::Location(v) => v,
            &Value::Register(v) => v as u32,
            &Value::Csr(v) => v,
            _ => panic!("{:?}", self),
        }
    }

    // For calc
    pub fn eval(&self) -> i64 {
        match self {
            &Value::General(v) => v,
            &Value::Location(v) => v as i64, // No sign extension
            &Value::Register(v) => v as i64, // Shouldn't this be an error?
            &Value::Csr(v) => v as i64, // ...
            _ => panic!("{:?}", self),
        }
    }

    pub fn get_string(&self) -> &String {
        match self {
            &Value::String(ref s) => s,
            _ => panic!("{:?}", self),
        }
    }

    pub fn get_strings(&self) -> &Vec<String> {
        match self {
            &Value::Strings(ref strings) => strings,
            _ => panic!("{:?}", self),
        }
    }

    pub fn get_integers(&self) -> &Vec<i64> {
        match self {
            &Value::Integers(ref xs) => xs,
            _ => panic!("{:?}", self),
        }
    }
}

