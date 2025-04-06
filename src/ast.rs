// src/ast.rs

#[derive(Debug, Clone)]
pub struct Program {
    pub name: String,
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Declaration {
    VariableDecl {
        names: Vec<String>,
        type_spec: Expression,
    },
    ConstDecl {
        name: String,
        type_name: String,
        value: Expression,
    },
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment {
        target: Variable,
        value: Expression,
    },
    IfElse {
        condition: Condition,
        if_branch: Vec<Statement>,
        else_branch: Vec<Statement>,
    },
    DoWhile {
        condition: Condition,
        body: Vec<Statement>,
    },
    For {
        var: String,
        start: Expression,
        end: Expression,
        step: Expression,
        body: Vec<Statement>,
    },
    Input {
        var: String,
    },
    Output {
        expressions: Vec<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    Var(Variable),
    Integer(i32),
    Float(f32),
    String(String),
    Type(String),
    ArrayType {
        type_name: String,
        size: i32,
    },
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    Not(Box<Expression>),
    Literal(Box<Expression>),  // Use Box to break the recursive definition
}
#[derive(Debug, Clone)]
pub enum Variable {
    Simple(String),
    Array {
        name: String,
        index: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Equal,
    NotEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum Condition {
    Expr(Expression),
}