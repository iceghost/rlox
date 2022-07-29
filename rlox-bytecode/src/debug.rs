use crate::chunk::{Chunk, Opcode};

use num_traits::FromPrimitive;

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    eprintln!("== {name} ==");

    let mut offset = 0;
    while offset < chunk.code().len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    eprint!("{offset:04} ");

    if offset > 0 && chunk.lines()[offset] == chunk.lines()[offset - 1] {
        eprint!("   | ");
    } else {
        eprint!("{:4} ", chunk.lines()[offset])
    }

    match Opcode::from_u8(chunk.code()[offset]) {
        Some(Opcode::Constant) => constant_instruction("OP_CONSTANT", chunk, offset),
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
