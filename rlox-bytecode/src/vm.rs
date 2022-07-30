use std::slice;

use num_traits::FromPrimitive;

use crate::{
    chunk::{Chunk, Opcode},
    compiler,
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
    pub fn new() -> Self {
        let stack = Vec::new();
        Self { stack }
    }

    pub fn intepret(&mut self, source: &str) -> Result<(), InterpretError> {
        todo!();
        // let chunk = if let Some(chunk) = compiler::compile(source) {
        //     chunk
        // } else {
        //     return Err(InterpretError::Compile);
        // };

        // let ip = chunk.code().iter();

        // let chunk_iter = ChunkIter::new(&chunk, ip);
        // self.run(chunk_iter)
    }

    #[inline]
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    #[inline]
    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn run(&mut self, mut iter: ChunkIter) -> Result<(), InterpretError> {
        macro_rules! binary_op {
            ($op:tt) => {{
                let a = self.pop();
                let b = self.pop();
                self.push(a $op b);
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
                Some(Opcode::Add) => binary_op!(+),
                Some(Opcode::Subtract) => binary_op!(-),
                Some(Opcode::Multiply) => binary_op!(*),
                Some(Opcode::Divide) => binary_op!(/),
                Some(Opcode::Negate) => {
                    let value = -self.pop();
                    self.push(value);
                }
                Some(Opcode::Return) => {
                    eprintln!("{}", self.pop());
                    return Ok(());
                }
                None => return Err(InterpretError::Runtime),
            }
        }
    }
}

#[derive(Debug)]
pub enum InterpretError {
    Compile,
    Runtime,
}
