use std::slice;

use num_traits::FromPrimitive;

use crate::{
    chunk::{Chunk, Opcode},
    compiler::{Compiler},
    debug::disassemble_instruction,
    value::Value,
};

struct ChunkIter<'a> {
    chunk: &'a Chunk,
    ip: slice::Iter<'a, u8>,
}

impl<'a> ChunkIter<'a> {
    #[inline]
    fn new(chunk: &'a Chunk, ip: slice::Iter<'a, u8>) -> Self {
        Self { chunk, ip }
    }

    #[inline]
    fn read_byte(&mut self) -> u8 {
        *self.ip.next().unwrap()
    }

    #[inline]
    fn read_constant(&mut self) -> Value {
        self.chunk.constants()[self.read_byte() as usize]
    }

    #[inline]
    fn as_inner(&self) -> &Chunk {
        self.chunk
    }

    #[inline]
    fn offset(&self) -> usize {
        self.chunk.code().len() - self.ip.len()
    }
}

#[derive(Default)]
pub struct VM {
    stack: Vec<Value>,
}

impl VM {
    pub fn intepret(&mut self, source: &str) -> Result<(), InterpretError> {
        let mut compiler = Compiler::default();

        if !compiler.compile(source) {
            return Err(InterpretError::Compile);
        };

        let chunk = compiler.current_chunk();
        let ip = chunk.code().iter();

        let chunk_iter = ChunkIter::new(chunk, ip);
        self.run(chunk_iter)
    }

    #[inline]
    fn push(&mut self, value: impl Into<Value>) {
        self.stack.push(value.into());
    }

    #[inline]
    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    #[inline]
    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - distance]
    }

    fn run(&mut self, mut iter: ChunkIter) -> Result<(), InterpretError> {
        macro_rules! binary_op {
            ($op:tt) => {{
                let a = self.peek(0);
                let b = self.peek(1);
                match (a.as_double(), b.as_double()) {
                    (Some(a), Some(b)) => {
                        self.pop();
                        self.pop();
                        self.push(a $op b);
                    }
                    _ => {
                        self.runtime_error(&iter, "Operands must be numbers.");
                    }
                }
            }};
        }

        loop {
            if cfg!(debug_assertions) {
                eprint!("          ");
                for value in &self.stack {
                    eprint!("[ {value} ]")
                }
                eprintln!();
                disassemble_instruction(iter.as_inner(), iter.offset());
            }

            match Opcode::from_u8(iter.read_byte()) {
                Some(Opcode::Constant) => {
                    let constant = iter.read_constant();
                    self.push(constant);
                }
                Some(Opcode::Not) => {
                    let result = !self.pop().is_truthy();
                    self.push(result);
                }
                Some(Opcode::Nil) => self.push(()),
                Some(Opcode::True) => self.push(true),
                Some(Opcode::False) => self.push(false),
                Some(Opcode::Equal) => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(a == b);
                }
                Some(Opcode::Greater) => binary_op!(>),
                Some(Opcode::Less) => binary_op!(<),
                Some(Opcode::Add) => binary_op!(+),
                Some(Opcode::Subtract) => binary_op!(-),
                Some(Opcode::Multiply) => binary_op!(*),
                Some(Opcode::Divide) => binary_op!(/),
                Some(Opcode::Negate) => {
                    if let Some(number) = self.peek(0).as_double() {
                        self.pop();
                        let value = -number;
                        self.push(value);
                    } else {
                        self.runtime_error(&iter, "Operand must be a number.");
                        return Err(InterpretError::Runtime);
                    }
                }
                Some(Opcode::Return) => {
                    eprintln!("{}", self.pop());
                    return Ok(());
                }
                None => return Err(InterpretError::Runtime),
            }
        }
    }

    fn runtime_error(&mut self, iter: &ChunkIter, message: &str) {
        eprintln!("{message}");
        let line = iter.offset();
        eprintln!("[line {line}] in script");
        self.stack.clear();
    }
}

#[derive(Debug)]
pub enum InterpretError {
    Compile,
    Runtime,
}
