use crate::ast::{Program, Statement, Expression, Variable, BinaryOp, Literal};
use crate::symbol_table::{SymbolTable, Value, EntityType};

pub struct Interpreter {
    pub symbol_table: SymbolTable,
}

impl Interpreter {
    pub fn new(symbol_table: SymbolTable) -> Self {
        Interpreter { symbol_table }
    }
    
    pub fn execute(&mut self, program: &Program) -> Result<(), String> {
        for statement in &program.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }
    
    fn execute_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Assignment { target, value } => {
                let evaluated = self.evaluate_expression(value)?;
                
                match target {
                    Variable::Simple(name) => {
                        self.symbol_table.update_value(name, evaluated)
                    },
                    Variable::Indexed { name, index } => {
                        // Handle array assignments
                        let idx_value = self.evaluate_expression(index)?;
                        if let Value::Int(i) = idx_value {
                            // You'll need to implement this method in your symbol table
                            self.symbol_table.update_array_element(name, i as usize, evaluated)
                        } else {
                            Err(format!("Array index must be an Int"))
                        }
                    }
                }
            },
            Statement::Output { expressions } => {
                // For output statements, just evaluate expressions
                for expr in expressions {
                    let _ = self.evaluate_expression(expr)?;
                }
                Ok(())
            },
            Statement::IfElse { condition, if_block, else_block } => {
                // Placeholder implementation for IfElse
                // TODO: Implement full logic
                Err(format!("IfElse statement not yet implemented"))
            },
            Statement::DoWhile { body, condition } => {
                // Placeholder implementation for DoWhile
                // TODO: Implement full logic
                Err(format!("DoWhile statement not yet implemented"))
            },
            Statement::For { init, condition, update, body } => {
                // Placeholder implementation for For
                // TODO: Implement full logic
                Err(format!("For statement not yet implemented"))
            },
            _ => {
                // Handle any other variant not explicitly covered
                Err(format!("Unsupported statement type"))
            }
        }
    }
    
    fn evaluate_expression(&self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Literal(literal) => {
                match literal {
                    Literal::Int(i) => Ok(Value::Int(*i)),
                    Literal::Float(f) => Ok(Value::Float(*f)),
                    Literal::String(s) => Ok(Value::String(s.clone())),
                    Literal::Boolean(b) => Ok(Value::Boolean(*b)),
                }
            },
            Expression::Var(var) => {
                match var {
                    Variable::Simple(name) => {
                        if let Some(entry) = self.symbol_table.lookup(name) {
                            Ok(entry.value.clone())
                        } else {
                            Err(format!("Undefined variable '{}'", name))
                        }
                    },
                    // Handle array access
                    Variable::Indexed { name, index } => {
                        // Similar to array element updates
                        let idx_value = self.evaluate_expression(index)?;
                        // Implementation depends on your symbol table structure
                        // ...
                        Ok(Value::Int(0)) // Placeholder
                    }
                }
            },
            Expression::Binary { left, op, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                
                match (left_val, op, right_val) {
                    (Value::Int(l), BinaryOp::Add, Value::Int(r)) => Ok(Value::Int(l + r)),
                    (Value::Int(l), BinaryOp::Multiply, Value::Int(r)) => Ok(Value::Int(l * r)),
                    
                    (Value::Float(l), BinaryOp::Add, Value::Float(r)) => Ok(Value::Float(l + r)),
                    (Value::Float(l), BinaryOp::Multiply, Value::Float(r)) => Ok(Value::Float(l * r)),
                    
                    // Mixed type operations
                    (Value::Int(l), BinaryOp::Add, Value::Float(r)) => Ok(Value::Float(l as f64 + r)),
                    (Value::Float(l), BinaryOp::Add, Value::Int(r)) => Ok(Value::Float(l + r as f64)),
                    (Value::Int(l), BinaryOp::Multiply, Value::Float(r)) => Ok(Value::Float(l as f64 * r)),
                    (Value::Float(l), BinaryOp::Multiply, Value::Int(r)) => Ok(Value::Float(l * r as f64)),
                    
                    // Add more operations as needed
                    _ => Err(format!("Unsupported operation between values"))
                }
            },
            // Handle other expression types
        }
    }
}