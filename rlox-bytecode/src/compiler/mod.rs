use crate::{
	chunk::{Chunk, Opcode},
	debug,
	scanner::{token::Ty, Scanner},
	value::Value,
	vm::VM,
};

mod parser;

#[derive(Default)]
pub struct Compiler<'a> {
	parser: parser::Parser<'a>,
	compiling_chunk: Chunk,
}

impl<'a> Compiler<'a> {
	pub fn compile(&mut self, source: &'a str, vm: &mut VM) -> bool {
		let mut scanner = Scanner::new(source);
		self.parser.advance(&mut scanner);
		self.expression(&mut scanner, vm);
		self.end();
		self.parser
			.consume(&mut scanner, Ty::Eof, "Expect end of expression.");

		!self.parser.had_error()
	}

	fn end(&mut self) {
		self.emit_bytes([Opcode::Return as u8]);
		if !self.parser.had_error() {
			debug::disassemble_chunk(self.current_chunk_mut(), "code");
		}
	}

	fn binary(&mut self, scanner: &mut Scanner<'a>, vm: &mut VM) {
		let operator = self.parser.previous().ty();
		let rule = get_rule(operator);
		self.parse_precedence(scanner, vm, rule.precedence.successor());

		match operator {
			Ty::BangEqual => self.emit_bytes([Opcode::Equal as u8, Opcode::Not as u8]),
			Ty::EqualEqual => self.emit_bytes([Opcode::Equal as u8]),
			Ty::Greater => self.emit_bytes([Opcode::Greater as u8]),
			Ty::GreaterEqual => self.emit_bytes([Opcode::Less as u8, Opcode::Not as u8]),
			Ty::Less => self.emit_bytes([Opcode::Less as u8]),
			Ty::LessEqual => self.emit_bytes([Opcode::Greater as u8, Opcode::Not as u8]),
			Ty::Plus => self.emit_bytes([Opcode::Add as u8]),
			Ty::Minus => self.emit_bytes([Opcode::Subtract as u8]),
			Ty::Star => self.emit_bytes([Opcode::Multiply as u8]),
			Ty::Slash => self.emit_bytes([Opcode::Divide as u8]),
			_ => unreachable!(),
		}
	}

	fn grouping(&mut self, scanner: &mut Scanner<'a>, vm: &mut VM) {
		self.expression(scanner, vm);
		self.parser
			.consume(scanner, Ty::RightParen, "Expect ')' after expression.");
	}

	fn unary(&mut self, scanner: &mut Scanner<'a>, vm: &mut VM) {
		let operator = self.parser.previous().ty();

		self.parse_precedence(scanner, vm, Precedence::Unary);

		match operator {
			Ty::Minus => self.emit_bytes([Opcode::Negate as u8]),
			Ty::Bang => self.emit_bytes([Opcode::Not as u8]),
			_ => unreachable!(),
		}
	}

	fn literal(&mut self, _: &mut Scanner<'a>, _: &mut VM) {
		match self.parser.previous().ty() {
			Ty::Nil => self.emit_bytes([Opcode::Nil as u8]),
			Ty::True => self.emit_bytes([Opcode::True as u8]),
			Ty::False => self.emit_bytes([Opcode::False as u8]),
			_ => unreachable!(),
		}
	}

	fn expression(&mut self, scanner: &mut Scanner<'a>, vm: &mut VM) {
		self.parse_precedence(scanner, vm, Precedence::Assignment);
	}

	fn parse_precedence(&mut self, scanner: &mut Scanner<'a>, vm: &mut VM, prec: Precedence) {
		self.parser.advance(scanner);
		let prefix_rule = get_rule(self.parser.previous().ty()).prefix;
		let prefix_rule = if let Some(prefix_rule) = prefix_rule {
			prefix_rule
		} else {
			self.parser.error("Expect expression");
			return;
		};

		prefix_rule(self, scanner, vm);

		while prec <= get_rule(self.parser.current().ty()).precedence {
			self.parser.advance(scanner);
			let infix_rule = get_rule(self.parser.previous().ty()).infix.unwrap();
			infix_rule(self, scanner, vm);
		}
	}

	fn number(&mut self, _: &mut Scanner<'a>, _: &mut VM) {
		let value = self.parser.previous().lexeme().parse::<f64>().unwrap();
		self.emit_constant(value);
	}

	fn string(&mut self, _: &mut Scanner<'a>, vm: &mut VM) {
		let token = self.parser.previous();
		let lexeme = token.lexeme();
		let copied_str = (&lexeme[1..lexeme.len() - 1]).to_owned();
		let obj = vm.allocate_string(copied_str);
		self.emit_constant(obj);
	}

	fn emit_constant(&mut self, value: impl Into<Value>) {
		let constant = self.current_chunk_mut().add_constant(value);
		let constant = if let Ok(constant) = constant.try_into() {
			constant
		} else {
			self.parser.error("Too many constants in one chunk.");
			0
		};
		self.emit_bytes([Opcode::Constant as u8, constant]);
	}

	fn emit_bytes<const N: usize>(&mut self, bytes: [u8; N]) {
		let line = self.parser.previous().line();
		for byte in bytes {
			self.current_chunk_mut().write(byte, line);
		}
	}

	#[inline]
	pub fn current_chunk(&self) -> &Chunk {
		&self.compiling_chunk
	}

	#[inline]
	fn current_chunk_mut(&mut self) -> &mut Chunk {
		&mut self.compiling_chunk
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Precedence {
	None,
	Assignment, // =
	Or,         // or
	And,        // and
	Equality,   // == !=
	Comparison, // < > <= >=
	Term,       // + -
	Factor,     // * /
	Unary,      // ! -
	Call,       // . ()
	Primary,
}

impl Precedence {
	pub fn successor(self) -> Self {
		match self {
			Precedence::None => Precedence::Assignment,
			Precedence::Assignment => Precedence::Or,
			Precedence::Or => Precedence::And,
			Precedence::And => Precedence::Equality,
			Precedence::Equality => Precedence::Comparison,
			Precedence::Comparison => Precedence::Term,
			Precedence::Term => Precedence::Factor,
			Precedence::Factor => Precedence::Unary,
			Precedence::Unary => Precedence::Call,
			Precedence::Call => Precedence::Primary,
			Precedence::Primary => Precedence::Primary,
		}
	}
}

type ParseFn<'a> = fn(&mut Compiler<'a>, &mut Scanner<'a>, &mut VM);

struct ParseRule<'a> {
	prefix: Option<ParseFn<'a>>,
	infix: Option<ParseFn<'a>>,
	precedence: Precedence,
}

impl<'a> ParseRule<'a> {
	fn new(
		prefix: Option<ParseFn<'a>>,
		infix: Option<ParseFn<'a>>,
		precedence: Precedence,
	) -> Self {
		Self {
			prefix,
			infix,
			precedence,
		}
	}
}

fn get_rule<'a>(operator: Ty) -> ParseRule<'a> {
	#[rustfmt::skip]
    let (prefix, infix, precedence): (Option<ParseFn>, Option<ParseFn>, Precedence) = match operator
    {
        Ty::LeftParen    => (Some(Compiler::grouping), None,                   Precedence::None),
        Ty::RightParen   => (None,                     None,                   Precedence::None),
        Ty::LeftBrace    => (None,                     None,                   Precedence::None),
        Ty::RightBrace   => (None,                     None,                   Precedence::None),
        Ty::Comma        => (None,                     None,                   Precedence::None),
        Ty::Dot          => (None,                     None,                   Precedence::None),
        Ty::Minus        => (Some(Compiler::unary),    Some(Compiler::binary), Precedence::Term),
        Ty::Plus         => (None,                     Some(Compiler::binary), Precedence::Term),
        Ty::Semicolon    => (None,                     None,                   Precedence::None),
        Ty::Slash        => (None,                     Some(Compiler::binary), Precedence::Factor),
        Ty::Star         => (None,                     Some(Compiler::binary), Precedence::Factor),
        Ty::Bang         => (Some(Compiler::unary),    None,                   Precedence::None),
        Ty::BangEqual    => (None,                     Some(Compiler::binary), Precedence::Equality),
        Ty::Equal        => (None,                     None,                   Precedence::None),
        Ty::EqualEqual   => (None,                     Some(Compiler::binary), Precedence::Equality),
        Ty::Greater      => (None,                     Some(Compiler::binary), Precedence::Comparison),
        Ty::GreaterEqual => (None,                     Some(Compiler::binary), Precedence::Comparison),
        Ty::Less         => (None,                     Some(Compiler::binary), Precedence::Comparison),
        Ty::LessEqual    => (None,                     Some(Compiler::binary), Precedence::Comparison),
        Ty::Identifier   => (None,                     None,                   Precedence::None),
        Ty::String       => (Some(Compiler::string),   None,                   Precedence::None),
        Ty::Number       => (Some(Compiler::number),   None,                   Precedence::None),
        Ty::And          => (None,                     None,                   Precedence::None),
        Ty::Class        => (None,                     None,                   Precedence::None),
        Ty::Else         => (None,                     None,                   Precedence::None),
        Ty::False        => (Some(Compiler::literal),  None,                   Precedence::None),
        Ty::For          => (None,                     None,                   Precedence::None),
        Ty::Fun          => (None,                     None,                   Precedence::None),
        Ty::If           => (None,                     None,                   Precedence::None),
        Ty::Nil          => (Some(Compiler::literal),  None,                   Precedence::None),
        Ty::Or           => (None,                     None,                   Precedence::None),
        Ty::Print        => (None,                     None,                   Precedence::None),
        Ty::Return       => (None,                     None,                   Precedence::None),
        Ty::Super        => (None,                     None,                   Precedence::None),
        Ty::This         => (None,                     None,                   Precedence::None),
        Ty::True         => (Some(Compiler::literal),  None,                   Precedence::None),
        Ty::Var          => (None,                     None,                   Precedence::None),
        Ty::While        => (None,                     None,                   Precedence::None),
        Ty::Error        => (None,                     None,                   Precedence::None),
        Ty::Eof          => (None,                     None,                   Precedence::None),
    };
	ParseRule::new(prefix, infix, precedence)
}
