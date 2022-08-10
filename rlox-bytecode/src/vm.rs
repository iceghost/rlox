use std::{any::Any, slice};

use crate::{
	chunk::{Chunk, Opcode},
	compiler::Compiler,
	debug::disassemble_instruction,
	table::Table,
	value::{ObjString, Object, Value},
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
	fn read_string(&mut self) -> ObjString {
		self.read_constant().as_objstring().unwrap()
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
	object: Option<Object<dyn Any>>,
	strings: Table<()>,
	globals: Table<Value>,
}

impl VM {
	pub fn intepret(&mut self, source: &str) -> Result<(), InterpretError> {
		let mut compiler = Compiler::new(self);

		if !compiler.compile(source) {
			return Err(InterpretError::Compile);
		};

		let chunk = compiler.into_chunk();
		crate::debug::disassemble_chunk(&chunk, "test");
		let ip = chunk.code().iter();

		let chunk_iter = ChunkIter::new(&chunk, ip);
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

	pub fn allocate_string(&mut self, data: String) -> ObjString {
		match self.strings.keys().find(|&&obj| *obj == *data) {
			Some(&obj) => obj,
			None => {
				let mut obj: ObjString = Object::new(data.into());
				obj.set_next(self.object);
				self.object = Some(obj.into());
				self.strings.insert(obj, ());
				obj
			}
		}
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
				if self.stack.is_empty() {
					eprint!("<empty stack>");
				}
				for value in &self.stack {
					eprint!("[ {value} ]")
				}
				eprintln!();
				disassemble_instruction(iter.as_inner(), iter.offset());
			}

			match Opcode::try_from(iter.read_byte()) {
				Ok(Opcode::Constant) => {
					let constant = iter.read_constant();
					self.push(constant);
				}
				Ok(Opcode::Not) => {
					let result = !self.pop().is_truthy();
					self.push(result);
				}
				Ok(Opcode::Nil) => self.push(()),
				Ok(Opcode::True) => self.push(true),
				Ok(Opcode::False) => self.push(false),
				Ok(Opcode::Pop) => {
					self.pop();
				}
				Ok(Opcode::GetGlobal) => {
					let name = iter.read_string();
					let value = if let Some(value) = self.globals.get(&name) {
						*value
					} else {
						self.runtime_error(&iter, &format!("Undefined variable '{}'", name));
						return Err(InterpretError::Runtime);
					};
					self.push(value);
				}
				Ok(Opcode::DefineGlobal) => {
					let name = iter.read_string();
					self.globals.insert(name, *self.peek(0));
					self.pop();
				}
				Ok(Opcode::SetGlobal) => {
					let name = iter.read_string();
					let value = *self.peek(0);
					if let Some(assignee) = self.globals.get_mut(&name) {
						*assignee = value;
					} else {
						self.runtime_error(&iter, &format!("Undefined variable '{}'", name));
						return Err(InterpretError::Runtime);
					};
				}
				Ok(Opcode::Equal) => {
					let a = self.pop();
					let b = self.pop();
					self.push(a == b);
				}
				Ok(Opcode::Greater) => binary_op!(>),
				Ok(Opcode::Less) => binary_op!(<),
				Ok(Opcode::Add) => {
					let a = self.peek(1);
					let b = self.peek(0);
					if let (Some(a), Some(b)) = (a.as_str(), b.as_str()) {
						let concatenated = [a, b].join("");
						let obj = self.allocate_string(concatenated);
						self.pop();
						self.pop();
						self.push(obj);
					} else if let (Some(a), Some(b)) = (a.as_double(), b.as_double()) {
						self.pop();
						self.pop();
						self.push(a + b);
					} else {
						self.runtime_error(&iter, "Operands must be numbers.");
					}
				}
				Ok(Opcode::Subtract) => binary_op!(-),
				Ok(Opcode::Multiply) => binary_op!(*),
				Ok(Opcode::Divide) => binary_op!(/),
				Ok(Opcode::Negate) => {
					if let Some(number) = self.peek(0).as_double() {
						self.pop();
						let value = -number;
						self.push(value);
					} else {
						self.runtime_error(&iter, "Operand must be a number.");
						return Err(InterpretError::Runtime);
					}
				}
				Ok(Opcode::Print) => {
					println!("{}", self.pop());
				}
				Ok(Opcode::Return) => {
					return Ok(());
				}
				Err(()) => return Err(InterpretError::Runtime),
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

impl Drop for VM {
	fn drop(&mut self) {
		let mut maybe_obj = self.object;
		while let Some(obj) = maybe_obj {
			maybe_obj = obj.next();
			obj.drop_inner();
		}
	}
}

#[derive(Debug)]
pub enum InterpretError {
	Compile,
	Runtime,
}
