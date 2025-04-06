// src/main.rs

use std::fs::File;
use std::io::Write;

mod ast;
mod lexer;
mod parser;

fn main() {
    let input = r#"
    MainPrgm L3_software;
    Var
    let x, y, z: Int;
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

    match parser::parse(input) {
        Ok(program) => {
            writeln!(output_file, "Successfully parsed program: {:?}", program)
                .expect("Unable to write to file");
        }
        Err(err) => {
            writeln!(output_file, "Error parsing program: {}", err)
                .expect("Unable to write to file");
        }
    }
}