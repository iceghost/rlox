use std::{
    io::{self, Write},
    process::exit,
};

use compiler::Compiler;
use vm::{InterpretError, VM};

mod chunk;
mod compiler;
mod debug;
mod scanner;
mod value;
mod vm;

fn main() {
    let mut args = std::env::args();
    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        run_file(&args.nth(1).unwrap());
    } else {
        eprintln!("Usage: clox [path]");
        exit(64);
    }
}

fn repl() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut vm = VM::default();
    loop {
        let mut line: String = String::new();
        print!("> ");
        stdout.flush().unwrap();
        match stdin.read_line(&mut line) {
            Ok(0) | Err(_) => {
                println!();
                break;
            }
            Ok(_) => {
                vm.intepret(&line).unwrap();
            }
        }
    }
}

fn run_file(path: &str) {
    let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Could not open file \"{path}\".");
        eprintln!("Error: {e:#?}");
        exit(74);
    });
    // compiler::compile(&source);
    let mut vm = VM::default();
    let result = vm.intepret(&source);
    match result {
        Ok(_) => (),
        Err(InterpretError::Compile) => exit(65),
        Err(InterpretError::Runtime) => exit(70),
    }
}
