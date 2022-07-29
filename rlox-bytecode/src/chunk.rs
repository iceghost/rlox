use num_derive::FromPrimitive;

use crate::value::{Value, Values};

#[derive(FromPrimitive)]
#[repr(u8)]
pub enum Opcode {
    Constant,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

#[derive(Default)]
pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<usize>,
    constants: Values,
}

impl Chunk {
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: impl Into<Value>) -> u8 {
        self.constants.write(value.into());
        self.constants.len() as u8 - 1
    }

    pub fn code(&self) -> &[u8] {
        self.code.as_ref()
    }

    pub fn constants(&self) -> &Values {
        &self.constants
    }

    pub fn lines(&self) -> &[usize] {
        self.lines.as_ref()
    }
}
