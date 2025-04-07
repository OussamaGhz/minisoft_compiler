// src/semantic_analyzer.rs

use crate::ast::{Program, Declaration, Statement, Expression, BinaryOp, Condition, Variable};
use crate::symbol_table::{SymbolTable, SymbolEntry, EntityType, DataType, Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub struct SemanticAnalyzer {
    pub symbol_table: SymbolTable,
    pub errors: Vec<SemanticError>,
    pub source_map: HashMap<String, (usize, usize)>, // Map identifiers to line and column
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            source_map: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program, source_map: HashMap<String, (usize, usize)>) -> Result<(), Vec<SemanticError>> {
        self.source_map = source_map;
        
        // Process declarations
        for decl in &program.declarations {
            self.process_declaration(decl);
        }
        
        // Process statements
        for stmt in &program.statements {
            self.process_statement(stmt);
        }
        
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn process_declaration(&mut self, decl: &Declaration) {
        match decl {
            Declaration::VariableDecl { names, type_spec } => {
                let data_type = self.get_data_type(type_spec);
                
                for name in names {
                    let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                    
                    match type_spec {
                        Expression::Type(_) => {
                            let entry = SymbolEntry {
                                name: name.clone(),
                                entity_type: EntityType::Variable,
                                data_type: data_type.clone(),
                                value: Value::Undefined,
                                line,
                                column,
                            };
                            
                            if let Err(e) = self.symbol_table.insert(entry) {
                                self.errors.push(SemanticError {
                                    message: e,
                                    line,
                                    column,
                                });
                            }
                        },
                        Expression::ArrayType { type_name: _, size } => {
                            let entry = SymbolEntry {
                                name: name.clone(),
                                entity_type: EntityType::Array { size: *size },
                                data_type: data_type.clone(),
                                value: Value::Undefined,
                                line,
                                column,
                            };
                            
                            if let Err(e) = self.symbol_table.insert(entry) {
                                self.errors.push(SemanticError {
                                    message: e,
                                    line,
                                    column,
                                });
                            }
                        },
                        _ => {} // Should not happen based on grammar
                    }
                }
            },
            Declaration::ConstDecl { name, type_name, value } => {
                let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                let data_type = match type_name.as_str() {
                    "Int" => DataType::Int,
                    "Float" => DataType::Float,
                    _ => {
                        self.errors.push(SemanticError {
                            message: format!("Unknown type: {}", type_name),
                            line,
                            column,
                        });
                        return;
                    }
                };
                
                // Evaluate constant value
                if let Some(const_value) = self.evaluate_constant(value) {
                    // Check type compatibility
                    match (&data_type, &const_value) {
                        (DataType::Int, Value::Int(_)) | (DataType::Float, Value::Float(_)) => {
                            // Types match, insert into symbol table
                            let entry = SymbolEntry {
                                name: name.clone(),
                                entity_type: EntityType::Constant,
                                data_type,
                                value: const_value,
                                line,
                                column,
                            };
                            
                            if let Err(e) = self.symbol_table.insert(entry) {
                                self.errors.push(SemanticError {
                                    message: e,
                                    line,
                                    column,
                                });
                            }
                        },
                        _ => {
                            self.errors.push(SemanticError {
                                message: format!("Type mismatch for constant '{}': expected {:?}, got {:?}", 
                                    name, data_type, const_value),
                                line,
                                column,
                            });
                        }
                    }
                } else {
                    self.errors.push(SemanticError {
                        message: format!("Could not evaluate constant value for '{}'", name),
                        line,
                        column,
                    });
                }
            }
        }
    }

    fn process_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Assignment { target, value } => {
                self.check_assignment(target, value);
            },
            Statement::IfElse { condition, if_branch, else_branch } => {
                self.check_condition(condition);
                
                for stmt in if_branch {
                    self.process_statement(stmt);
                }
                
                for stmt in else_branch {
                    self.process_statement(stmt);
                }
            },
            Statement::DoWhile { condition, body } => {
                self.check_condition(condition);
                
                for stmt in body {
                    self.process_statement(stmt);
                }
            },
            Statement::For { var, start, end, step, body } => {
                // Check if variable exists
                if self.symbol_table.lookup(var).is_none() {
                    let (line, column) = self.source_map.get(var).unwrap_or(&(0, 0)).clone();
                    self.errors.push(SemanticError {
                        message: format!("Undeclared identifier: '{}'", var),
                        line,
                        column,
                    });
                }
                
                // Check expressions
                self.check_expression(start);
                self.check_expression(end);
                self.check_expression(step);
                
                // Process body
                for stmt in body {
                    self.process_statement(stmt);
                }
            },
            Statement::Input { var } => {
                if self.symbol_table.lookup(var).is_none() {
                    let (line, column) = self.source_map.get(var).unwrap_or(&(0, 0)).clone();
                    self.errors.push(SemanticError {
                        message: format!("Undeclared identifier: '{}'", var),
                        line,
                        column,
                    });
                }
            },
            Statement::Output { expressions } => {
                for expr in expressions {
                    self.check_expression(expr);
                }
            }
        }
    }

    fn check_assignment(&mut self, target: &Variable, value: &Expression) {
        match target {
            Variable::Simple(name) => {
                // Check if variable exists
                let entry = match self.symbol_table.lookup(name) {
                    Some(entry) => entry,
                    None => {
                        let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                        self.errors.push(SemanticError {
                            message: format!("Undeclared identifier: '{}'", name),
                            line,
                            column,
                        });
                        return;
                    }
                };
                
                // Check if assigning to constant
                if let EntityType::Constant = entry.entity_type {
                    let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                    self.errors.push(SemanticError {
                        message: format!("Cannot modify constant: '{}'", name),
                        line,
                        column,
                    });
                    return;
                }
                
                // Check expression
                self.check_expression(value);
                
                // Type checking would be done here, but we'd need type inference first
                // For now, we'll just make sure the expression is valid
            },
            Variable::Array { name, index } => {
                // Check if array exists
                let entry = match self.symbol_table.lookup(name) {
                    Some(entry) => entry,
                    None => {
                        let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                        self.errors.push(SemanticError {
                            message: format!("Undeclared identifier: '{}'", name),
                            line,
                            column,
                        });
                        return;
                    }
                };
                
                // Check if it's an array
                match &entry.entity_type {
                    EntityType::Array { size } => {
                        // Check index
                        if let Some(idx_val) = self.evaluate_constant(index) {
                            if let Value::Int(idx) = idx_val {
                                if idx < 0 || idx >= *size {
                                    let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                                    self.errors.push(SemanticError {
                                        message: format!("Array index out of bounds: '{}[{}]', size is {}", 
                                            name, idx, size),
                                        line,
                                        column,
                                    });
                                }
                            }
                        } else {
                            // If we can't evaluate the index at compile time, we need runtime checks
                            self.check_expression(index);
                        }
                    },
                    _ => {
                        let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                        self.errors.push(SemanticError {
                            message: format!("'{}' is not an array", name),
                            line,
                            column,
                        });
                    }
                }
                
                // Check expression
                self.check_expression(value);
            }
        }
    }

    fn check_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Var(var) => {
                match var {
                    Variable::Simple(name) => {
                        if self.symbol_table.lookup(name).is_none() {
                            let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                            self.errors.push(SemanticError {
                                message: format!("Undeclared identifier: '{}'", name),
                                line,
                                column,
                            });
                        }
                    },
                    Variable::Array { name, index } => {
                        if let Some(entry) = self.symbol_table.lookup(name) {
                            if let EntityType::Array { size } = entry.entity_type {
                                // Check index bounds if possible
                                if let Some(idx_val) = self.evaluate_constant(index) {
                                    if let Value::Int(idx) = idx_val {
                                        if idx < 0 || idx >= size {
                                            let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                                            self.errors.push(SemanticError {
                                                message: format!("Array index out of bounds: '{}[{}]', size is {}", 
                                                    name, idx, size),
                                                line,
                                                column,
                                            });
                                        }
                                    }
                                }
                                // Check the index expression
                                self.check_expression(index);
                            } else {
                                let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                                self.errors.push(SemanticError {
                                    message: format!("'{}' is not an array", name),
                                    line,
                                    column,
                                });
                            }
                        } else {
                            let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                            self.errors.push(SemanticError {
                                message: format!("Undeclared identifier: '{}'", name),
                                line,
                                column,
                            });
                        }
                    }
                }
            },
            Expression::Binary { left, op, right } => {
                self.check_expression(left);
                self.check_expression(right);
                
                // Check for division by zero
                if let BinaryOp::Divide = op {
                    if let Some(right_val) = self.evaluate_constant(right) {
                        match right_val {
                            Value::Int(0) | Value::Float(0.0) => {
                                // For simplicity, we're using 0,0 as position since we don't have the actual position
                                self.errors.push(SemanticError {
                                    message: "Division by zero".to_string(),
                                    line: 0,
                                    column: 0,
                                });
                            },
                            _ => {}
                        }
                    }
                }
                
                // Type checking would be more extensive here
            },
            Expression::Not(expr) => {
                self.check_expression(expr);
            },
            _ => {
                // Other expression types are literals or types, no need to check
            }
        }
    }

    fn check_condition(&mut self, condition: &Condition) {
        match condition {
            Condition::Expr(expr) => {
                self.check_expression(expr);
            }
        }
    }

    fn get_data_type(&self, type_expr: &Expression) -> DataType {
        match type_expr {
            Expression::Type(type_name) => {
                match type_name.as_str() {
                    "Int" => DataType::Int,
                    "Float" => DataType::Float,
                    _ => DataType::Int, // Default, but should not happen
                }
            },
            Expression::ArrayType { type_name, size: _ } => {
                match type_name.as_str() {
                    "Int" => DataType::Int,
                    "Float" => DataType::Float,
                    _ => DataType::Int, // Default, but should not happen
                }
            },
            _ => DataType::Int, // Default, but should not happen
        }
    }

    fn evaluate_constant(&self, expr: &Expression) -> Option<Value> {
        match expr {
            Expression::Integer(n) => Some(Value::Int(*n)),
            Expression::Float(n) => Some(Value::Float(*n)),
            Expression::Binary { left, op, right } => {
                if let (Some(left_val), Some(right_val)) = (self.evaluate_constant(left), self.evaluate_constant(right)) {
                    match (left_val, right_val) {
                        (Value::Int(left_int), Value::Int(right_int)) => {
                            match op {
                                BinaryOp::Add => Some(Value::Int(left_int + right_int)),
                                BinaryOp::Subtract => Some(Value::Int(left_int - right_int)),
                                BinaryOp::Multiply => Some(Value::Int(left_int * right_int)),
                                BinaryOp::Divide => {
                                    if right_int == 0 {
                                        // Division by zero is caught in another check
                                        None
                                    } else {
                                        Some(Value::Int(left_int / right_int))
                                    }
                                },
                                _ => None // Logical operators not supported in constant evaluation
                            }
                        },
                        (Value::Float(left_float), Value::Float(right_float)) => {
                            match op {
                                BinaryOp::Add => Some(Value::Float(left_float + right_float)),
                                BinaryOp::Subtract => Some(Value::Float(left_float - right_float)),
                                BinaryOp::Multiply => Some(Value::Float(left_float * right_float)),
                                BinaryOp::Divide => {
                                    if right_float == 0.0 {
                                        // Division by zero is caught in another check
                                        None
                                    } else {
                                        Some(Value::Float(left_float / right_float))
                                    }
                                },
                                _ => None // Logical operators not supported in constant evaluation
                            }
                        },
                        _ => None // Mixed types not supported in constant evaluation
                    }
                } else {
                    None
                }
            },
            Expression::Not(_) => None, // Not supported in constant evaluation
            Expression::Var(var) => {
                match var {
                    Variable::Simple(name) => {
                        if let Some(entry) = self.symbol_table.lookup(name) {
                            if let EntityType::Constant = entry.entity_type {
                                Some(entry.value.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    },
                    _ => None
                }
            },
            _ => None,
        }
    }
}