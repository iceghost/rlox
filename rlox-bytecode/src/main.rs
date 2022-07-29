use chunk::{Chunk, Opcode};
use debug::disassemble_chunk;

mod chunk;
mod debug;
mod value;

fn main() {
    let mut chunk = Chunk::default();
    let constant = chunk.add_constant(1.2.into());
    chunk.write(Opcode::Constant as u8, 123);
    chunk.write(constant, 123);
    chunk.write(Opcode::Return as u8, 123);
    disassemble_chunk(&chunk, "test chunk");
}
