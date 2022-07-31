use crate::chunk::{Chunk, Opcode};

use num_traits::FromPrimitive;

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

    match Opcode::from_u8(chunk.code()[offset]) {
        Some(Opcode::Constant) => constant_instruction("OP_CONSTANT", chunk, offset),
        Some(Opcode::Nil) => simple_instruction("OP_NIL", offset),
        Some(Opcode::True) => simple_instruction("OP_TRUE", offset),
        Some(Opcode::False) => simple_instruction("OP_FALSE", offset),
        Some(Opcode::Equal) => simple_instruction("OP_EQUAL", offset),
        Some(Opcode::Greater) => simple_instruction("OP_GREATER", offset),
        Some(Opcode::Less) => simple_instruction("OP_LESS", offset),
        Some(Opcode::Add) => simple_instruction("OP_ADD", offset),
        Some(Opcode::Subtract) => simple_instruction("OP_SUBTRACT", offset),
        Some(Opcode::Multiply) => simple_instruction("OP_MULTIPLY", offset),
        Some(Opcode::Divide) => simple_instruction("OP_DIVIDE", offset),
        Some(Opcode::Not) => simple_instruction("OP_NOT", offset),
        Some(Opcode::Negate) => simple_instruction("OP_NEGATE", offset),
        Some(Opcode::Return) => simple_instruction("OP_RETURN", offset),
        None => {
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
