// src/symbol_table.rs

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Variable,
    Constant,
    Array { size: i32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Int,
    Float,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Float(f32),
    Undefined,
}

#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub name: String,
    pub entity_type: EntityType,
    pub data_type: DataType,
    pub value: Value,
    pub line: usize,
    pub column: usize,
}

pub struct SymbolTable {
    pub table: HashMap<String, SymbolEntry>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            table: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entry: SymbolEntry) -> Result<(), String> {
        if self.table.contains_key(&entry.name) {
            Err(format!("Semantic Error: Double declaration of '{}' at line {}, column {}", 
                entry.name, entry.line, entry.column))
        } else {
            self.table.insert(entry.name.clone(), entry);
            Ok(())
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        self.table.get(name)
    }
}