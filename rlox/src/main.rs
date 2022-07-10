use std::{borrow::Cow, io::BufRead, process::exit};

use interpreter::{Interpreter, RuntimeError};
use parser::{ParseError, Parser};
use scanner::{ScanError, Scanner};
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
    had_input_error: bool,
    had_runtime_error: bool,
}

impl Lox {
    fn run_file(&mut self, path: String) {
        let program =
            std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("failed to open {}", path));
        self.run(program);

        if self.had_input_error {
            exit(65);
        }

        if self.had_runtime_error {
            exit(70);
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
            self.had_input_error = false;
            self.had_runtime_error = false;
        }
    }

    fn run(&mut self, source: String) {
        let scanner = Scanner::new(source);

        let tokens = match scanner.scan_tokens() {
            Ok(tokens) => tokens,
            Err(err) => {
                self.had_input_error = true;
                return self.scan_error(err);
            }
        };

        let mut parser = Parser::new(tokens);

        let expr = match parser.parse() {
            Ok(expr) => expr,
            Err(err) => {
                self.had_input_error = true;
                return self.parse_error(err);
            }
        };

        let interpreter = Interpreter;

        match interpreter.interpret(&expr) {
            Ok(_) => {}
            Err(err) => {
                self.had_runtime_error = true;
                self.runtime_error(err)
            }
        }
    }

    fn scan_error(&mut self, err: ScanError) {
        match err {
            ScanError::Custom(line, message) => {
                self.report(line, "".into(), message);
            }
            ScanError::Multiple(errs) => {
                for err in errs {
                    self.scan_error(err);
                }
            }
        }
    }

    fn parse_error(&mut self, err: ParseError) {
        match err {
            ParseError::Custom(token, message) => {
                if token.ty == TokenTy::Eof {
                    self.report(token.line, " at end".into(), message);
                } else {
                    self.report(
                        token.line,
                        format!(" at '{}'", token.lexeme).into(),
                        message,
                    );
                }
            }
        }
    }

    fn runtime_error(&mut self, error: RuntimeError) {
        match error {
            RuntimeError::Custom(token, message) => {
                eprintln!("{message}\n[line {}]", token.line);
            }
        }
        self.had_runtime_error = true;
    }

    fn report(&mut self, line: usize, location: Cow<'_, str>, message: Cow<'_, str>) {
        eprintln!("[line {}] Error {}: {}", line, location, message);
        self.had_input_error = true;
    }
}
