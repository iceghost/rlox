use num_traits::FromPrimitive;

use crate::{
    chunk::{Chunk, Opcode},
    debug::disassemble_instruction,
    value::Value,
};

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: std::slice::Iter<'a, u8>,
    stack: Vec<Value>,
}

impl<'a> VM<'a> {
    fn new(chunk: &'a Chunk) -> Self {
        let ip = chunk.code().iter();
        let stack = Vec::new();
        Self { chunk, ip, stack }
    }

    pub fn intepret(chunk: &'a Chunk) -> Result<(), InterpretError> {
        let mut vm = Self::new(chunk);
        vm.run()
    }

    #[inline]
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    #[inline]
    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    #[inline]
    fn read_byte(&mut self) -> u8 {
        *self.ip.next().unwrap()
    }

    #[inline]
    fn read_constant(&mut self) -> Value {
        self.chunk.constants()[self.read_byte() as usize]
    }

    fn run(&mut self) -> Result<(), InterpretError> {
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
                disassemble_instruction(self.chunk, self.chunk.code().len() - self.ip.len());
            }

            match Opcode::from_u8(self.read_byte()) {
                Some(Opcode::Constant) => {
                    let constant = self.read_constant();
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
