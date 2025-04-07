// src/main.rs

use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

mod ast;
mod lexer;
mod parser;
mod symbol_table;  // Add these new modules
mod semantic_analyzer;

use crate::semantic_analyzer::SemanticAnalyzer;

fn main() {
    let input = r#"
    MainPrgm L3_software;
    Var
    let x, y, z, i: Int;
    @define Const Software: Int = 91;
    let A, B: [Int; 10];
    
    BeginPg
    {
        for i from 1 to 10 step 2 {
            x := 5 + (-3);
            if (x > 0) then {
                output("Value", x);
            } else {
                input(y);
            }
        }
        
        do {
            z := z + 1;
        } while (z < 100);
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
                    },
                    _ => {}
                }
            }
            
            // Run semantic analysis
            let mut analyzer = SemanticAnalyzer::new();
            match analyzer.analyze(&program, source_map) {
                Ok(_) => {
                    writeln!(output_file, "Semantic analysis successful. Symbol table:").expect("Unable to write to file");
                    for (name, entry) in &analyzer.symbol_table.table {
                        writeln!(output_file, "{}: {:?}", name, entry).expect("Unable to write to file");
                    }
                },
                Err(errors) => {
                    writeln!(output_file, "Semantic errors:").expect("Unable to write to file");
                    for error in errors {
                        writeln!(output_file, "Line {}, Column {}: {}", 
                            error.line, error.column, error.message).expect("Unable to write to file");
                    }
                }
            }
        },
        Err(err) => {
            writeln!(output_file, "Error parsing program: {}", err)
                .expect("Unable to write to file");
        }
    }
}