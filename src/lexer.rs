// src/lexer.rs
use logos::Logos;

#[derive(Debug, Logos, PartialEq, Clone)]
pub enum Token {
    // Keywords
    #[token("MainPrgm")]
    KwMainPrgm,
    #[token("Var")]
    KwVar,
    #[token("BeginPg")]
    KwBeginPg,
    #[token("EndPg")]
    KwEndPg,
    #[token("let")]
    KwLet,
    #[token("if")]
    KwIf,
    #[token("else")]
    KwElse,
    #[token("do")]
    KwDo,
    #[token("while")]
    KwWhile,
    #[token("for")]
    KwFor,
    #[token("from")]
    KwFrom,
    #[token("to")]
    KwTo,
    #[token("step")]
    KwStep,
    #[token("input")]
    KwInput,
    #[token("output")]
    KwOutput,
    #[token("@define")]
    KwDefine,
    #[token("Const")]
    KwConst,
    #[token("Int")]
    KwInt,
    #[token("Float")]
    KwFloat,

    // Identifiers (added with validation)
    #[regex(r"[a-zA-Z][a-zA-Z0-9_]*", |lex| {
    let s = lex.slice();
    if s.len() > 14 || s.ends_with('_') || s.contains("__") {
        Err(())
    } else {
        Ok(s.to_string())
    }
})]
    Ident(String),

    // Numeric Literals
    #[regex(r"\d+|\([+-]?\d+\)", |lex| {
      let s = lex.slice().trim_matches(|c| c == '(' || c == ')');
      s.parse::<i32>().ok()
  })]
    IntLiteral(i32),

    #[regex(r"\d+\.\d+|\([+-]?\d+\.\d+\)", |lex| {
      let s = lex.slice().trim_matches(|c| c == '(' || c == ')');
      s.parse::<f64>().ok()
  })]
    FloatLiteral(f64),

    // === Comments (skipped) ===
    #[regex(r"<!-[^-!>]*-!>", logos::skip)] // Single-line comment
    #[regex(r"\{--(?s:[^-]|[-][^-])*--\}", logos::skip)] // Multi-line comment

    // Punctuation
    #[token(";")]
    Semi,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(":=")]
    Assign,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)] // Skip whitespace
    Error,
}
