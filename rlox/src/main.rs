use std::{io::BufRead, process::exit};

use interpreter::{RuntimeError, Interpreter};
use parser::Parser;
use scanner::Scanner;
use token::Token;
use token_type::TokenTy;
mod ast_printer;
mod expr;
mod interpreter;
mod literal;
mod object;
mod parser;
mod scanner;
mod token;
mod token_type;

fn main() {
    let mut args = std::env::args();
    if args.len() > 1 {
        println!("Usage: rslox [script]")
    }
    args.next(); // first arg is program name, e.g rslox
    let mut lox = Lox::default();
    match args.next() {
        Some(arg) => lox.run_file(arg),
        None => lox.run_prompt(),
    }
}

#[derive(Default)]
struct Lox {
    had_error: bool,
    had_runtime_error: bool,
}

impl Lox {
    fn run_file(&mut self, path: String) {
        let program =
            std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("failed to open {}", path));
        self.run(program);

        if self.had_error {
            exit(65);
        }
    }

    fn run_prompt(&mut self) {
        let mut reader = std::io::BufReader::new(std::io::stdin());
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).expect("failed to read line") == 0 {
                break;
            }
            self.run(line);
            self.had_error = false;
        }
    }

    fn run(&mut self, source: String) {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);

        let expr = parser.parse().unwrap();

        if self.had_error {
            return;
        }

        let mut interpreter = Interpreter;

        interpreter.interpret(&expr);
    }

    fn error(&mut self, token: &Token, message: &str) {
        if token.ty == TokenTy::Eof {
            Self::report(token.line, " at end", message);
        } else {
            Self::report(token.line, &format!(" at '{}'", token.lexeme), message);
        }
        self.had_error = true;
    }

    fn runtime_error(&mut self, error: RuntimeError) {
        match error {
            RuntimeError::Custom(token, message) => {
                println!("{message}\n[line {}]", token.line);
            }
        }
        self.had_runtime_error = true;
    }

    fn report(line: usize, location: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, location, message);
    }
}
