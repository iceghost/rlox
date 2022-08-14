use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::value::{Value, Values};

#[derive(FromPrimitive)]
#[repr(u8)]
pub enum Opcode {
	Constant,
	Nil,
	True,
	False,
	Pop,
	GetLocal,
	GetGlobal,
	DefineGlobal,
	SetLocal,
	SetGlobal,
	Equal,
	Greater,
	Less,
	Add,
	Subtract,
	Multiply,
	Divide,
	Not,
	Negate,
	Print,
	Jump,
	JumpIfFalse,
	Loop,
	Return,
}

impl TryFrom<u8> for Opcode {
	type Error = ();

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match Opcode::from_u8(value) {
			Some(opcode) => Ok(opcode),
			None => Err(()),
		}
	}
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

	pub fn add_constant(&mut self, value: impl Into<Value>) -> usize {
		self.constants.write(value.into());
		self.constants.len() - 1
	}

	#[inline]
	pub fn code(&self) -> &[u8] {
		self.code.as_ref()
	}

	#[inline ]
	pub fn code_mut(&mut self) -> &mut [u8] {
		self.code.as_mut()
	}

	#[inline]
	pub fn constants(&self) -> &Values {
		&self.constants
	}

	#[inline]
	pub fn lines(&self) -> &[usize] {
		self.lines.as_ref()
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.code.len()
	}
}
