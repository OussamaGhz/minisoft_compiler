// src/semantic_analyzer.rs

use crate::ast::{BinaryOp, Condition, Declaration, Expression, Program, Statement, Variable};
use crate::symbol_table::{DataType, EntityType, SymbolEntry, SymbolTable, Value};
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

    pub fn analyze(
        &mut self,
        program: &Program,
        source_map: HashMap<String, (usize, usize)>,
    ) -> Result<(), Vec<SemanticError>> {
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
                        }
                        Expression::ArrayType { type_name: _, size } => {
                            let initial_values = vec![Value::Undefined; *size as usize];
                            let entry = SymbolEntry {
                                name: name.clone(),
                                entity_type: EntityType::Array { size: *size },
                                data_type: data_type.clone(),
                                value: Value::Array(initial_values),
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
                        }
                        _ => {} // Should not happen based on grammar
                    }
                }
            }
            Declaration::ConstDecl {
                name,
                type_name,
                value,
            } => {
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
                        }
                        _ => {
                            self.errors.push(SemanticError {
                                message: format!(
                                    "Type mismatch for constant '{}': expected {:?}, got {:?}",
                                    name, data_type, const_value
                                ),
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
            }
            Statement::IfElse {
                condition,
                if_branch,
                else_branch,
            } => {
                self.check_condition(condition);

                for stmt in if_branch {
                    self.process_statement(stmt);
                }

                for stmt in else_branch {
                    self.process_statement(stmt);
                }
            }
            Statement::DoWhile { condition, body } => {
                self.check_condition(condition);

                for stmt in body {
                    self.process_statement(stmt);
                }
            }
            Statement::For {
                var,
                start,
                end,
                step,
                body,
            } => {
                // Check if variable exists
                if self.symbol_table.lookup(var).is_none() {
                    let (line, column) = self.source_map.get(var).unwrap_or(&(0, 0)).clone();
                    self.errors.push(SemanticError {
                        message: format!("Undeclared identifier: '{}'", var),
                        line,
                        column,
                    });
                } else {
                    // Initialize loop variable with start value if possible
                    if let Some(start_val) = self.evaluate_expression(start) {
                        if let Err(e) = self.symbol_table.update_value(var, start_val) {
                            let (line, column) =
                                self.source_map.get(var).unwrap_or(&(0, 0)).clone();
                            self.errors.push(SemanticError {
                                message: e,
                                line,
                                column,
                            });
                        }
                    }
                }

                // Check expressions
                self.check_expression(start);
                self.check_expression(end);
                self.check_expression(step);

                // Process body
                for stmt in body {
                    self.process_statement(stmt);
                }
            }
            Statement::Input { var } => {
                if self.symbol_table.lookup(var).is_none() {
                    let (line, column) = self.source_map.get(var).unwrap_or(&(0, 0)).clone();
                    self.errors.push(SemanticError {
                        message: format!("Undeclared identifier: '{}'", var),
                        line,
                        column,
                    });
                } else {
                    // For input statements, mark the variable as having a runtime value
                    // We can't know what the value will be at compile time
                    // But we should mark that it's been assigned
                    if let Err(e) = self.symbol_table.update_value(var, Value::Undefined) {
                        let (line, column) = self.source_map.get(var).unwrap_or(&(0, 0)).clone();
                        self.errors.push(SemanticError {
                            message: e,
                            line,
                            column,
                        });
                    }
                }
            }
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

                 // Check for string literals in assignment
                 if let Expression::String(s) = value {
                    let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                    self.errors.push(SemanticError {
                        message: format!("Cannot assign string '{}' to variable '{}' of type {:?}", s, name, entry.data_type),
                        line,
                        column,
                    });
                    return;
                }

                // Check expression
                self.check_expression(value);

                // Special case for literal values - handle them directly
                match value {
                    Expression::Integer(n) => {
                        let val = Value::Int(*n);
                        if let Err(e) = self.symbol_table.update_value(name, val) {
                            let (line, column) =
                                self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                            self.errors.push(SemanticError {
                                message: e,
                                line,
                                column,
                            });
                        }
                        return;
                    }
                    Expression::Float(n) => {
                        let val = Value::Float(*n);
                        if let Err(e) = self.symbol_table.update_value(name, val) {
                            let (line, column) =
                                self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                            self.errors.push(SemanticError {
                                message: e,
                                line,
                                column,
                            });
                        }
                        return;
                    }
                    Expression::Literal(expr) => {
                        // Handle unwrapping the literal value
                        match &**expr {
                            Expression::Integer(n) => {
                                let val = Value::Int(*n);
                                if let Err(e) = self.symbol_table.update_value(name, val) {
                                    let (line, column) =
                                        self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                                    self.errors.push(SemanticError {
                                        message: e,
                                        line,
                                        column,
                                    });
                                }
                                return;
                            }
                            Expression::Float(n) => {
                                let val = Value::Float(*n);
                                if let Err(e) = self.symbol_table.update_value(name, val) {
                                    let (line, column) =
                                        self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                                    self.errors.push(SemanticError {
                                        message: e,
                                        line,
                                        column,
                                    });
                                }
                                return;
                            }
                            _ => {} // Fall through to the general case
                        }
                    }
                    _ => {} // Fall through to the general case
                }
                

                // Try to evaluate the expression and update the symbol table
                if let Some(evaluated_value) = self.evaluate_expression(value) {
                    // Here we could add type checking between entry.data_type and evaluated_value
                    // For now, just update the value
                    if let Err(e) = self.symbol_table.update_value(name, evaluated_value) {
                        let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                        self.errors.push(SemanticError {
                            message: e,
                            line,
                            column,
                        });
                    }
                } else {
                    // If we can't evaluate at compile time, mark as having a runtime value
                    if let Err(e) = self.symbol_table.update_value(name, Value::Undefined) {
                        let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                        self.errors.push(SemanticError {
                            message: e,
                            line,
                            column,
                        });
                    }
                }
            }
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
                if let EntityType::Array { size } = entry.entity_type {
                    // Evaluate the index expression
                    if let Some(Value::Int(idx)) = self.evaluate_constant(index) {
                        if idx < 0 || idx >= size {
                            let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                            self.errors.push(SemanticError {
                                message: format!("Array index out of bounds: '{}[{}]', size is {}", name, idx, size),
                                line,
                                column,
                            });
                        } else {
                            // Evaluate the value expression
                            if let Some(value) = self.evaluate_expression(value) {
                                // Update the array element at idx
                                if let Err(e) = self.symbol_table.update_array_element(name, idx as usize, value) {
                                    let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                                    self.errors.push(SemanticError {
                                        message: e,
                                        line,
                                        column,
                                    });
                                }
                            } else {
                                // Mark element as Undefined if value can't be determined
                                self.symbol_table.update_array_element(name, idx as usize, Value::Undefined).ok();
                            }
                        }
                    } else {
                        // Index isn't a constant; check expressions but can't track value
                        self.check_expression(index);
                    }
                } else {
                    let (line, column) = self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                    self.errors.push(SemanticError {
                        message: format!("'{}' is not an array", name),
                        line,
                        column,
                    });
                }
            
                // Check the value expression
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
                            let (line, column) =
                                self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                            self.errors.push(SemanticError {
                                message: format!("Undeclared identifier: '{}'", name),
                                line,
                                column,
                            });
                        }
                    }
                    Variable::Array { name, index } => {
                        if let Some(entry) = self.symbol_table.lookup(name) {
                            if let EntityType::Array { size } = entry.entity_type {
                                // Check index bounds if possible
                                if let Some(idx_val) = self.evaluate_constant(index) {
                                    if let Value::Int(idx) = idx_val {
                                        if idx < 0 || idx >= size {
                                            let (line, column) = self
                                                .source_map
                                                .get(name)
                                                .unwrap_or(&(0, 0))
                                                .clone();
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
                                let (line, column) =
                                    self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                                self.errors.push(SemanticError {
                                    message: format!("'{}' is not an array", name),
                                    line,
                                    column,
                                });
                            }
                        } else {
                            let (line, column) =
                                self.source_map.get(name).unwrap_or(&(0, 0)).clone();
                            self.errors.push(SemanticError {
                                message: format!("Undeclared identifier: '{}'", name),
                                line,
                                column,
                            });
                        }
                    }
                }
            }
            Expression::Binary { left, op, right } => {
                self.check_expression(left);
                self.check_expression(right);

                // Check for division by zero using evaluate_expression to track variable values
                if let BinaryOp::Divide = op {
                    if let Some(right_val) = self.evaluate_expression(right) {
                        match right_val {
                            Value::Int(0) | Value::Float(0.0) => {
                                // Get the source position from the right expression if possible
                                let (line, column) = self.get_expr_source_pos(right);
                                self.errors.push(SemanticError {
                                    message: "Division by zero".to_string(),
                                    line,
                                    column,
                                });
                            }
                            _ => {}
                        }
                    }
                }

                // Type checking would be more extensive here
            }
            Expression::Not(expr) => {
                self.check_expression(expr);
            }
            _ => {
                // Other expression types are literals or types, no need to check
            }
        }
    }


    /// Helper to get the source position of an expression (simplified)
    fn get_expr_source_pos(&self, expr: &Expression) -> (usize, usize) {
        match expr {
            Expression::Var(var) => {
                match var {
                    Variable::Simple(name) => *self.source_map.get(name).unwrap_or(&(0, 0)),
                    Variable::Array { name, .. } => *self.source_map.get(name).unwrap_or(&(0, 0)),
                }
            }
            // For literals, use default (0,0) as their positions aren't tracked
            _ => (0, 0),
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
            }
            Expression::ArrayType { type_name, size: _ } => {
                match type_name.as_str() {
                    "Int" => DataType::Int,
                    "Float" => DataType::Float,
                    _ => DataType::Int, // Default, but should not happen
                }
            }
            _ => DataType::Int, // Default, but should not happen
        }
    }

    // This function is similar to evaluate_constant but handles more cases
    // and is used to track runtime values in the symbol table
    fn evaluate_expression(&self, expr: &Expression) -> Option<Value> {
        match expr {
            Expression::Integer(n) => Some(Value::Int(*n)),
            Expression::Float(n) => Some(Value::Float(*n)),
            Expression::String(_) => None, // We're not tracking string values in this example
            Expression::Literal(inner_expr) => {
                // Unwrap the literal and evaluate the inner expression
                self.evaluate_expression(inner_expr)
            }
            Expression::Var(var) => {
                match var {
                    Variable::Simple(name) => {
                        if let Some(entry) = self.symbol_table.lookup(name) {
                            match &entry.value {
                                Value::Undefined => None, // Value not determined at compile time
                                _ => Some(entry.value.clone()),
                            }
                        } else {
                            None
                        }
                    }
                    Variable::Array { name: _, index: _ } => {
                        // For array access, we'd need to track individual elements
                        // For simplicity, we'll just return None for array elements
                        None
                    }
                }
            }
            Expression::Binary { left, op, right } => {
                if let (Some(left_val), Some(right_val)) = (
                    self.evaluate_expression(left),
                    self.evaluate_expression(right),
                ) {
                    match (left_val, right_val) {
                        // The binary operation handling remains the same
                        (Value::Int(left_int), Value::Int(right_int)) => {
                            // Implementation for integer operations remains the same
                            match op {
                                BinaryOp::Add => Some(Value::Int(left_int + right_int)),
                                // Other operations remain the same
                                _ => None,
                            }
                        }
                        (Value::Float(left_float), Value::Float(right_float)) => {
                            // Implementation for float operations remains the same
                            match op {
                                BinaryOp::Add => Some(Value::Float(left_float + right_float)),
                                // Other operations remain the same
                                _ => None,
                            }
                        }
                        // Handle mixed types (Int and Float)
                        (Value::Int(left_int), Value::Float(right_float)) => {
                            // Convert int to float and perform float operation
                            let left_float = left_int as f32;
                            match op {
                                BinaryOp::Add => Some(Value::Float(left_float + right_float)),
                                BinaryOp::Subtract => Some(Value::Float(left_float - right_float)),
                                BinaryOp::Multiply => Some(Value::Float(left_float * right_float)),
                                BinaryOp::Divide => {
                                    if right_float == 0.0 {
                                        None // Division by zero
                                    } else {
                                        Some(Value::Float(left_float / right_float))
                                    }
                                }
                                // Other operations would use similar logic
                                _ => None,
                            }
                        }
                        (Value::Float(left_float), Value::Int(right_int)) => {
                            // Convert int to float and perform float operation
                            let right_float = right_int as f32;
                            match op {
                                BinaryOp::Add => Some(Value::Float(left_float + right_float)),
                                BinaryOp::Subtract => Some(Value::Float(left_float - right_float)),
                                BinaryOp::Multiply => Some(Value::Float(left_float * right_float)),
                                BinaryOp::Divide => {
                                    if right_int == 0 {
                                        None // Division by zero
                                    } else {
                                        Some(Value::Float(left_float / right_float))
                                    }
                                }
                                // Other operations would use similar logic
                                _ => None,
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            Expression::Not(expr) => {
                // Logic for the Not operator remains the same
                if let Some(val) = self.evaluate_expression(expr) {
                    match val {
                        Value::Int(i) => Some(Value::Int(if i == 0 { 1 } else { 0 })),
                        Value::Float(f) => Some(Value::Int(if f == 0.0 { 1 } else { 0 })),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn evaluate_constant(&self, expr: &Expression) -> Option<Value> {
        match expr {
            Expression::Integer(n) => Some(Value::Int(*n)),
            Expression::Float(n) => Some(Value::Float(*n)),
            Expression::Literal(inner_expr) => {
                // Unwrap the literal and evaluate the inner expression
                self.evaluate_constant(inner_expr)
            }
            Expression::Binary { left, op, right } => {
                if let (Some(left_val), Some(right_val)) =
                    (self.evaluate_constant(left), self.evaluate_constant(right))
                {
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
                                }
                                _ => None, // Logical operators not supported in constant evaluation
                            }
                        }
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
                                }
                                _ => None, // Logical operators not supported in constant evaluation
                            }
                        }
                        _ => None, // Mixed types not supported in constant evaluation
                    }
                } else {
                    None
                }
            }
            Expression::Not(_) => None, // Not supported in constant evaluation
            Expression::Var(var) => match var {
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
                }
                _ => None,
            },
            _ => None,
        }
    }
}
