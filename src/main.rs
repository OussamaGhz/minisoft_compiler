// src/main.rs

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;


mod ast;
mod lexer;
mod parser;
mod semantic_analyzer;
mod symbol_table; // Add these new modules

use crate::semantic_analyzer::SemanticAnalyzer;

fn main() {
    let input = r#"
 MainPrgm SimpleTest;
Var
let a, b, c: Int;
@define Const PI: Float = 3.14;

let result: Int;

BeginPg
{
    a := 10;
    c := 20;
    b := 2;
    result := a / b;
    output("The result is:", result);
}
EndPg;
    "#;

    let mut output_file = File::create("output.txt").expect("Unable to create output file");

    // Parse the program
    match parser::parse(input) {
        Ok(program) => {
            writeln!(output_file, "Successfully parsed program: {:?}", program)
                .expect("Unable to write to file");

            // Build source map for identifiers
            let mut source_map = HashMap::new();
            let tokens = lexer::lex(input);

            for token in &tokens {
                match &token.token {
                    lexer::Token::Identifier(name) => {
                        source_map.insert(name.clone(), (token.line, token.column));
                    }
                    _ => {}
                }
            }

            // Run semantic analysis
            let mut analyzer = SemanticAnalyzer::new();
            match analyzer.analyze(&program, source_map) {
                Ok(_) => {
                    writeln!(
                        output_file,
                        "Semantic analysis successful.\n\nSymbol Table:"
                    )
                    .expect("Unable to write to file");
                    writeln!(output_file, "{}", analyzer.symbol_table.format_table())
                        .expect("Unable to write to file");
                }
                Err(errors) => {
                    writeln!(output_file, "Semantic errors:").expect("Unable to write to file");
                    for error in errors {
                        writeln!(
                            output_file,
                            "Line {}, Column {}: {}",
                            error.line, error.column, error.message
                        )
                        .expect("Unable to write to file");
                    }
                }
            }
        }
        Err(err) => {
            writeln!(output_file, "Error parsing program: {}", err)
                .expect("Unable to write to file");
        }
    }
}
