use chunk::{Chunk, Opcode};
use vm::VM;

mod chunk;
mod debug;
mod value;
mod vm;

fn main() {
    let mut chunk = Chunk::default();

    let constant = chunk.add_constant(1.2);
    chunk.write(Opcode::Constant as u8, 123);
    chunk.write(constant, 123);

    let constant = chunk.add_constant(3.4);
    chunk.write(Opcode::Constant as u8, 123);
    chunk.write(constant, 123);

    chunk.write(Opcode::Add as u8, 123);

    let constant = chunk.add_constant(5.6);
    chunk.write(Opcode::Constant as u8, 123);
    chunk.write(constant, 123);

    chunk.write(Opcode::Divide as u8, 123);
    chunk.write(Opcode::Negate as u8, 123);

    chunk.write(Opcode::Return as u8, 123);
    // debug::disassemble_chunk(&chunk, "test chunk");
    VM::intepret(&chunk).unwrap();
}
