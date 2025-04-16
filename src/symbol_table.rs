// src/symbol_table.rs

use std::collections::HashMap;
use std::fmt::Write;

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

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i32),
    Float(f32),
    Array(Vec<Value>),
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
    pub fn update_value(&mut self, name: &str, value: Value) -> Result<(), String> {
        if let Some(entry) = self.table.get_mut(name) {
            entry.value = value;
            Ok(())
        } else {
            Err(format!("Cannot update undefined variable '{}'", name))
        }
    }
    pub fn new() -> Self {
        SymbolTable {
            table: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entry: SymbolEntry) -> Result<(), String> {
        if self.table.contains_key(&entry.name) {
            Err(format!(
                "Semantic Error: Double declaration of '{}' at line {}, column {}",
                entry.name, entry.line, entry.column
            ))
        } else {
            self.table.insert(entry.name.clone(), entry);
            Ok(())
        }
    }

    pub fn update_array_element(
        &mut self,
        name: &str,
        index: usize,
        value: Value,
    ) -> Result<(), String> {
        if let Some(entry) = self.table.get_mut(name) {
            match &mut entry.value {
                Value::Array(elements) => {
                    if index < elements.len() {
                        elements[index] = value;
                        Ok(())
                    } else {
                        Err(format!(
                            "Index {} out of bounds for array '{}' of size {}",
                            index,
                            name,
                            elements.len()
                        ))
                    }
                }
                _ => Err(format!("'{}' is not an array", name)),
            }
        } else {
            Err(format!("Undeclared array '{}'", name))
        }
    }

    
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        self.table.get(name)
    }

    pub fn format_table(&self) -> String {
        let mut output = String::new();
        writeln!(output, "+{:-<20}+{:-<15}+{:-<10}+{:-<15}+{:-<8}+{:-<8}+", 
            "", "", "", "", "", "").unwrap();
        writeln!(output, "| {:<18} | {:<13} | {:<8} | {:<13} | {:<6} | {:<6} |", 
            "Name", "Entity Type", "Type", "Value", "Line", "Column").unwrap();
        writeln!(output, "+{:-<20}+{:-<15}+{:-<10}+{:-<15}+{:-<8}+{:-<8}+", 
            "", "", "", "", "", "").unwrap();
        
        let mut entries: Vec<&SymbolEntry> = self.table.values().collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        
        for entry in entries {
            let entity_type = match &entry.entity_type {
                EntityType::Variable => "Variable".to_string(),
                EntityType::Constant => "Constant".to_string(),
                EntityType::Array { size } => format!("Array[{}]", size),
            };
            
            let value_str = match &entry.value {
                Value::Int(i) => i.to_string(),
                Value::Float(f) => f.to_string(),
                Value::Array(elements) => {
                    let elements_str: Vec<String> = elements.iter().map(|v| {
                        match v {
                            Value::Int(n) => n.to_string(),
                            Value::Float(n) => format!("{:.1}", n),
                            Value::Undefined => "-".to_string(),
                            Value::Array(_) => "[]".to_string(),
                        }
                    }).collect();
                    format!("[{}]", elements_str.join(", "))
                },
                Value::Undefined => "-".to_string(),
            };
    
            writeln!(output, "| {:<18} | {:<13} | {:<8} | {:<13} | {:<6} | {:<6} |", 
                entry.name, 
                entity_type, 
                format!("{:?}", entry.data_type), 
                value_str, 
                entry.line, 
                entry.column
            ).unwrap();
        }
        
        writeln!(output, "+{:-<20}+{:-<15}+{:-<10}+{:-<15}+{:-<8}+{:-<8}+", 
            "", "", "", "", "", "").unwrap();
        output
    }
}
