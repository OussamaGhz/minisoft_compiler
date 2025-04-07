// src/quadruple.rs

#[derive(Debug, Clone)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Assign,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Equal,
    NotEqual,
    And,
    Or,
    Not,
    Goto,
    IfTrue,
    IfFalse,
    Label,
    Input,
    Output,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Variable(String),
    Constant(String),
    ArrayElement(String, Box<Operand>),
    Temp(usize),
    Label(usize),
    StringLiteral(String),
}

#[derive(Debug, Clone)]
pub struct Quadruple {
    pub operator: Operator,
    pub arg1: Option<Operand>,
    pub arg2: Option<Operand>,
    pub result: Option<Operand>,
}

pub struct QuadrupleGenerator {
    pub quads: Vec<Quadruple>,
    pub temp_count: usize,
    pub label_count: usize,
}

impl QuadrupleGenerator {
    pub fn new() -> Self {
        QuadrupleGenerator {
            quads: Vec::new(),
            temp_count: 0,
            label_count: 0,
        }
    }
    
    pub fn new_temp(&mut self) -> Operand {
        let temp = Operand::Temp(self.temp_count);
        self.temp_count += 1;
        temp
    }
    
    pub fn new_label(&mut self) -> Operand {
        let label = Operand::Label(self.label_count);
        self.label_count += 1;
        label
    }
    
    pub fn emit(&mut self, operator: Operator, arg1: Option<Operand>, arg2: Option<Operand>, result: Option<Operand>) {
        self.quads.push(Quadruple {
            operator,
            arg1,
            arg2,
            result,
        });
    }
    
    // Here you would implement methods to generate quads for each AST node type
    // For example:
    
    pub fn generate_from_program(&mut self, program: &crate::ast::Program) {
        // Generate quads for declarations
        for decl in &program.declarations {
            self.generate_from_declaration(decl);
        }
        
        // Generate quads for statements
        for stmt in &program.statements {
            self.generate_from_statement(stmt);
        }
    }
    
    fn generate_from_declaration(&mut self, decl: &crate::ast::Declaration) {
        // Implementation would depend on how you want to handle declarations
        // Generally, variables don't need quadruples, but initializations might
    }
    
    fn generate_from_statement(&mut self, stmt: &crate::ast::Statement) {
        match stmt {
            crate::ast::Statement::Assignment { target, value } => {
                // Generate code for the expression
                let expr_result = self.generate_from_expression(value);
                
                // Create the assignment quadruple
                let target_operand = match target {
                    crate::ast::Variable::Simple(name) => Some(Operand::Variable(name.clone())),
                    crate::ast::Variable::Array { name, index } => {
                        let index_result = self.generate_from_expression(index);
                        Some(Operand::ArrayElement(name.clone(), Box::new(index_result.unwrap())))
                    }
                };
                
                self.emit(Operator::Assign, expr_result, None, target_operand);
            },
            // Add cases for other statement types
            _ => {
                // Implement other statement types (if-else, loops, etc.)
            }
        }
    }
    
    fn generate_from_expression(&mut self, expr: &crate::ast::Expression) -> Option<Operand> {
        match expr {
            crate::ast::Expression::Var(var) => {
                match var {
                    crate::ast::Variable::Simple(name) => Some(Operand::Variable(name.clone())),
                    crate::ast::Variable::Array { name, index } => {
                        let index_result = self.generate_from_expression(index);
                        Some(Operand::ArrayElement(name.clone(), Box::new(index_result.unwrap())))
                    }
                }
            },
            crate::ast::Expression::Integer(n) => Some(Operand::Constant(n.to_string())),
            crate::ast::Expression::Float(n) => Some(Operand::Constant(n.to_string())),
            crate::ast::Expression::String(s) => Some(Operand::StringLiteral(s.clone())),
            crate::ast::Expression::Binary { left, op, right } => {
                let left_result = self.generate_from_expression(left).unwrap();
                let right_result = self.generate_from_expression(right).unwrap();
                let result = self.new_temp();
                
                let operator = match op {
                    crate::ast::BinaryOp::Add => Operator::Add,
                    crate::ast::BinaryOp::Subtract => Operator::Subtract,
                    crate::ast::BinaryOp::Multiply => Operator::Multiply,
                    crate::ast::BinaryOp::Divide => Operator::Divide,
                    crate::ast::BinaryOp::LessThan => Operator::LessThan,
                    crate::ast::BinaryOp::GreaterThan => Operator::GreaterThan,
                    crate::ast::BinaryOp::LessEqual => Operator::LessEqual,
                    crate::ast::BinaryOp::GreaterEqual => Operator::GreaterEqual,
                    crate::ast::BinaryOp::Equal => Operator::Equal,
                    crate::ast::BinaryOp::NotEqual => Operator::NotEqual,
                    crate::ast::BinaryOp::And => Operator::And,
                    crate::ast::BinaryOp::Or => Operator::Or,
                };
                
                self.emit(operator, Some(left_result), Some(right_result), Some(result.clone()));
                Some(result)
            },
            crate::ast::Expression::Not(expr) => {
                let expr_result = self.generate_from_expression(expr).unwrap();
                let result = self.new_temp();
                
                self.emit(Operator::Not, Some(expr_result), None, Some(result.clone()));
                Some(result)
            },
            _ => None
        }
    }
}