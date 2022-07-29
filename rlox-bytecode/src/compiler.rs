use crate::scanner::{token, Scanner};

pub fn compile(source: &str) {
    let mut scanner = Scanner::new(source);
    let mut line = 0;
    loop {
        let token = scanner.scan_token();
        if token.line() != line {
            eprint!("{:4} ", token.line());
            line = token.line();
        } else {
            eprint!("   | ");
        }
        eprintln!("{:16} '{}'", format!("{:?}", token.ty()), token.lexeme().escape_debug());

        if token.ty() == token::Ty::Eof {
            break;
        }
    }
}
