mod lexer;
use lexer::Token;
use logos::Logos; // Import the Logos trait

fn main() {
    let input = r#"
      MainPrgm
      let x := (-5);
      if (y >= 10.5) AND (z != 0) {
        output("Hello");
      }
      <!- This is a comment -!>
      {-- Another
         comment --}
    "#;

    let mut lex = Token::lexer(input);

    while let Some(tok) = lex.next() {
        match tok {
            Token::Error => println!("Error at {:?}", lex.span()),
            _ => println!("{:?} at {:?}", tok, lex.span()),
        }
    }
}
