use logos::{Logos, Span};
use std::fmt;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    #[token("MainPrgm")]
    MainPrgm,

    #[token("Var")]
    Var,

    #[token("BeginPg")]
    BeginPg,

    #[token("EndPg")]
    EndPg,

    #[token("let")]
    Let,

    #[token("Int")]
    Int,

    #[token("Float")]
    Float,

    #[token("@define")]
    Define,

    #[token("Const")]
    Const,

    #[token("input")]
    Input,

    #[token("output")]
    Output,

    #[token("if")]
    If,

    #[token("then")]
    Then,

    #[token("else")]
    Else,

    #[token("do")]
    Do,

    #[token("while")]
    While,

    #[token("for")]
    For,

    #[token("from")]
    From,

    #[token("to")]
    To,

    #[token("step")]
    Step,

    // Operators
    #[token("+")]
    Plus,

    #[token("=")]
    Equals,

    #[token("-")]
    Minus,

    #[token("*")]
    Multiply,

    #[token("/")]
    Divide,

    #[token("<")]
    LessThan,

    #[token(">")]
    GreaterThan,

    #[token("<=")]
    LessEqual,

    #[token(">=")]
    GreaterEqual,

    #[token("==")]
    Equal,

    #[token("!=")]
    NotEqual,

    #[token("AND")]
    And,

    #[token("OR")]
    Or,

    #[token("!")]
    Not,

    // Separators and punctuation
    #[token(":=")]
    Assign,

    #[token(";")]
    Semicolon,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    // Literals
    #[regex(r"[a-zA-Z]([a-zA-Z0-9_]*[a-zA-Z0-9])?", lex_identifier)]
    Identifier(String),

    #[regex(r"[0-9]+", lex_integer)]
    IntLiteral(i32),

    #[regex(r"\([+-][0-9]+\)", lex_signed_integer)]
    SignedIntLiteral(i32),

    #[regex(r"[0-9]+\.[0-9]+", lex_float)]
    FloatLiteral(f32),

    #[regex(r"\([+-][0-9]+\.[0-9]+\)", lex_signed_float)]
    SignedFloatLiteral(f32),

    #[regex(r#""[^"]*""#, lex_string)]
    StringLiteral(String),

     // Comments - fixed regex patterns
     #[regex(r"//[^\n]*", logos::skip)]            // Single-line comments with //
     #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip)] // Multi-line comments with /* ... */
     
     // Whitespace and newlines
     #[regex(r"[ \t\n\r]+", logos::skip)]
     // Error variant
     Error,
}

// Helper functions for token conversion
fn lex_identifier(lex: &mut logos::Lexer<Token>) -> String {
    lex.slice().to_string()
}

fn lex_integer(lex: &mut logos::Lexer<Token>) -> i32 {
    lex.slice().parse().unwrap_or(0)
}

fn lex_signed_integer(lex: &mut logos::Lexer<Token>) -> i32 {
    let text = lex.slice();
    // Extract the sign and number from the parentheses format: "(+123)" or "(-123)"
    let number_str = &text[1..text.len() - 1]; // Remove the parentheses
    number_str.parse().unwrap_or(0)
}

fn lex_float(lex: &mut logos::Lexer<Token>) -> f32 {
    lex.slice().parse().unwrap_or(0.0)
}

fn lex_signed_float(lex: &mut logos::Lexer<Token>) -> f32 {
    let text = lex.slice();
    // Extract the sign and number from the parentheses format: "(+123.45)" or "(-123.45)"
    let number_str = &text[1..text.len() - 1]; // Remove the parentheses
    number_str.parse().unwrap_or(0.0)
}

fn lex_string(lex: &mut logos::Lexer<Token>) -> String {
    let text = lex.slice();
    // Remove the surrounding quotes
    text[1..text.len() - 1].to_string()
}

// Implement Display for Token to pretty-print tokens
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "Identifier({})", s),
            Token::IntLiteral(n) => write!(f, "IntLiteral({})", n),
            Token::SignedIntLiteral(n) => write!(f, "SignedIntLiteral({})", n),
            Token::FloatLiteral(n) => write!(f, "FloatLiteral({})", n),
            Token::SignedFloatLiteral(n) => write!(f, "SignedFloatLiteral({})", n),
            Token::StringLiteral(s) => write!(f, "StringLiteral({})", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug)]
pub struct LexResult {
    pub token: Token,
    pub span: Span,
    #[allow(dead_code)]
    pub line: usize,
    #[allow(dead_code)]
    pub column: usize,
}

pub fn lex(input: &str) -> Vec<LexResult> {
    let mut lexer = Token::lexer(input);
    let mut tokens = Vec::new();
    let mut line_starts = vec![0];
    
    // Build line starts index for column calculation
    for (i, c) in input.char_indices() {
        if c == '\n' {
            line_starts.push(i + 1);
        }
    }

    while let Some(token_result) = lexer.next() {
        match token_result {
            Ok(token) if token != Token::Error => {
                let span = lexer.span();
                
                // Calculate line and column of the token
                let mut l = 1;
                while l < line_starts.len() && line_starts[l] <= span.start {
                    l += 1;
                }
                let token_line = l;
                let token_column = span.start - line_starts[l - 1] + 1;
                
                tokens.push(LexResult {
                    token: token.clone(),
                    span,
                    line: token_line,
                    column: token_column,
                });
            }
            _ => {}
        }
    }

    tokens
}