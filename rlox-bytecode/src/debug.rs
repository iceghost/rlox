use crate::chunk::{Chunk, Opcode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
	eprintln!("== {name} ==");

	let mut offset = 0;
	while offset < chunk.code().len() {
		offset = disassemble_instruction(chunk, offset);
	}
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
	eprint!("{offset:04} ");

	if offset > 0 && chunk.lines()[offset] == chunk.lines()[offset - 1] {
		eprint!("   | ");
	} else {
		eprint!("{:4} ", chunk.lines()[offset])
	}

	match Opcode::try_from(chunk.code()[offset]) {
		Ok(Opcode::Constant) => constant_instruction("OP_CONSTANT", chunk, offset),
		Ok(Opcode::Nil) => simple_instruction("OP_NIL", offset),
		Ok(Opcode::True) => simple_instruction("OP_TRUE", offset),
		Ok(Opcode::False) => simple_instruction("OP_FALSE", offset),
		Ok(Opcode::Pop) => simple_instruction("OP_POP", offset),
		Ok(Opcode::GetLocal) => byte_instruction("OP_GET_LOCAL", chunk, offset),
		Ok(Opcode::GetGlobal) => constant_instruction("OP_GET_GLOBAL", chunk, offset),
		Ok(Opcode::DefineGlobal) => constant_instruction("OP_DEFINE_GLOBAL", chunk, offset),
		Ok(Opcode::SetLocal) => byte_instruction("OP_SET_LOCAL", chunk, offset),
		Ok(Opcode::SetGlobal) => constant_instruction("OP_SET_GLOBAL", chunk, offset),
		Ok(Opcode::Equal) => simple_instruction("OP_EQUAL", offset),
		Ok(Opcode::Greater) => simple_instruction("OP_GREATER", offset),
		Ok(Opcode::Less) => simple_instruction("OP_LESS", offset),
		Ok(Opcode::Add) => simple_instruction("OP_ADD", offset),
		Ok(Opcode::Subtract) => simple_instruction("OP_SUBTRACT", offset),
		Ok(Opcode::Multiply) => simple_instruction("OP_MULTIPLY", offset),
		Ok(Opcode::Divide) => simple_instruction("OP_DIVIDE", offset),
		Ok(Opcode::Not) => simple_instruction("OP_NOT", offset),
		Ok(Opcode::Negate) => simple_instruction("OP_NEGATE", offset),
		Ok(Opcode::Print) => simple_instruction("OP_PRINT", offset),
		Ok(Opcode::Jump) => jump_instruction("OP_JUMP", 1, chunk, offset),
		Ok(Opcode::JumpIfFalse) => jump_instruction("OP_JUMP_IF_FALSE", 1, chunk, offset),
		Ok(Opcode::Loop) => jump_instruction("OP_LOOP", -1, chunk, offset),
		Ok(Opcode::Return) => simple_instruction("OP_RETURN", offset),
		Err(()) => {
			eprintln!("Unknown opcode {}", chunk.code()[offset]);
			offset + 1
		}
	}
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
	let constant = chunk.code()[offset + 1] as usize;
	eprintln!("{name:-16} {constant:4} '{}'", chunk.constants()[constant]);
	offset + 2
}

fn simple_instruction(name: &str, offset: usize) -> usize {
	eprintln!("{name}");
	offset + 1
}

fn byte_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
	let slot = chunk.code()[offset + 1];
	eprintln!("{name:-16} {slot:4}");
	offset + 2
}

fn jump_instruction(name: &str, sign: isize, chunk: &Chunk, offset: usize) -> usize {
	let jump = ((chunk.code()[offset + 1] as u16) << 8) | chunk.code()[offset + 2] as u16;
	eprintln!(
		"{name:-16} {offset:4} -> {}",
		offset as isize + 3 + sign * jump as isize
	);
	offset + 3
}
