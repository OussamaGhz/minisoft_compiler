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
 MainPrgm ArrayAndLoopTest;
Var
let n, sum, i: Int;
@define Const SIZE: Int = 10;
let numbers: [Int; 10];

BeginPg
{ // coment
    n := 5;
    sum := 0;
    
    for i from 0 to SIZE-1 step 1 {
        numbers[i] := i * i;
    }
    
    for i from 0 to n step 1 {
        if (numbers[i] > 10) then {
            sum := sum + numbers[i];
            output("Added", numbers[i], "to sum");
        } else {
            output("Skipped", numbers[i]);
        }
    }
    
    output("Final sum:", sum);
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
