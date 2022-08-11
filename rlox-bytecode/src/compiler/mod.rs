use crate::{
	chunk::{Chunk, Opcode},
	debug,
	scanner::token::Ty,
	value::Value,
	vm::VM,
};

mod parser;

use self::parser::Parser;

pub struct Compilation<'a> {
	parser: Parser<'a>,
	compiling_chunk: Chunk,
	vm: &'a mut VM,
}

impl<'a> Compilation<'a> {
	pub fn new(vm: &'a mut VM, source: &'a str) -> Self {
		let parser = Parser::new(source);
		let compiling_chunk = Chunk::default();
		Self {
			parser,
			compiling_chunk,
			vm,
		}
	}

	pub fn execute(&mut self) -> bool {
		while !self.parser.matches(Ty::Eof) {
			self.declaration();
		}
		self.end();
		self.parser.consume(Ty::Eof, "Expect end of expression.");

		!self.parser.had_error()
	}

	fn end(&mut self) {
		self.emit_bytes([Opcode::Return as u8]);
		if self.parser.had_error() {
			debug::disassemble_chunk(self.current_chunk_mut(), "code");
		}
	}

	fn declaration(&mut self) {
		if self.parser.matches(Ty::Var) {
			self.var_declaration();
		} else {
			self.statement();
		}

		if self.parser.panic_mode() {
			self.parser.synchronize();
		}
	}

	fn var_declaration(&mut self) {
		let global = self.parse_variable("Expect variable name.");
		if self.parser.matches(Ty::Equal) {
			self.expression();
		} else {
			self.emit_bytes([Opcode::Nil as u8]);
		};
		self.parser
			.consume(Ty::Semicolon, "Expect ';' after variable declaration.");
		self.define_variable(global);
	}

	fn parse_variable(&mut self, error_message: &'static str) -> u8 {
		self.parser.consume(Ty::Identifier, error_message);
		self.identifier_constant(self.parser.previous().lexeme())
	}

	fn identifier_constant(&mut self, name: &str) -> u8 {
		let obj = self.vm.allocate_string(name.to_owned());
		self.make_constant(obj)
	}

	fn define_variable(&mut self, global: u8) {
		self.emit_bytes([Opcode::DefineGlobal as u8, global]);
	}

	fn statement(&mut self) {
		if self.parser.matches(Ty::Print) {
			self.print_statement();
		} else {
			self.expression_statement();
		}
	}

	fn print_statement(&mut self) {
		self.expression();
		self.parser
			.consume(Ty::Semicolon, "Expect ';' after value.");
		self.emit_bytes([Opcode::Print as u8])
	}

	fn expression_statement(&mut self) {
		self.expression();
		self.parser
			.consume(Ty::Semicolon, "Expect ';' after expression.");
		self.emit_bytes([Opcode::Pop as u8])
	}

	fn binary(&mut self, _: bool) {
		let operator = self.parser.previous().ty();
		let rule = get_rule(operator);
		self.parse_precedence(rule.precedence.successor());

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

	fn grouping(&mut self, _: bool) {
		self.expression();
		self.parser
			.consume(Ty::RightParen, "Expect ')' after expression.");
	}

	fn unary(&mut self, _: bool) {
		let operator = self.parser.previous().ty();

		self.parse_precedence(Precedence::Unary);

		match operator {
			Ty::Minus => self.emit_bytes([Opcode::Negate as u8]),
			Ty::Bang => self.emit_bytes([Opcode::Not as u8]),
			_ => unreachable!(),
		}
	}

	fn literal(&mut self, _: bool) {
		match self.parser.previous().ty() {
			Ty::Nil => self.emit_bytes([Opcode::Nil as u8]),
			Ty::True => self.emit_bytes([Opcode::True as u8]),
			Ty::False => self.emit_bytes([Opcode::False as u8]),
			_ => unreachable!(),
		}
	}

	fn expression(&mut self) {
		self.parse_precedence(Precedence::Assignment);
	}

	fn parse_precedence(&mut self, prec: Precedence) {
		self.parser.advance();
		let prefix_rule = get_rule(self.parser.previous().ty()).prefix;
		let prefix_rule = if let Some(prefix_rule) = prefix_rule {
			prefix_rule
		} else {
			self.parser.error("Expect expression");
			return;
		};

		let can_assign = prec <= Precedence::Assignment;
		prefix_rule(self, can_assign);

		while prec <= get_rule(self.parser.current().ty()).precedence {
			self.parser.advance();
			let infix_rule = get_rule(self.parser.previous().ty()).infix.unwrap();
			infix_rule(self, can_assign);
		}
	}

	fn number(&mut self, _: bool) {
		let value = self.parser.previous().lexeme().parse::<f64>().unwrap();
		self.emit_constant(value);
	}

	fn string(&mut self, _: bool) {
		let token = self.parser.previous();
		let lexeme = token.lexeme();
		let copied_str = (&lexeme[1..lexeme.len() - 1]).to_owned();
		let obj = self.vm.allocate_string(copied_str);
		self.emit_constant(obj);
	}

	fn variable(&mut self, can_assign: bool) {
		self.named_variable(self.parser.previous().lexeme(), can_assign);
	}

	fn named_variable(&mut self, name: &str, can_assign: bool) {
		let arg = self.identifier_constant(name);
		if can_assign && self.parser.matches(Ty::Equal) {
			self.expression();
			self.emit_bytes([Opcode::SetGlobal as u8, arg]);
		} else {
			self.emit_bytes([Opcode::GetGlobal as u8, arg]);
		}
	}

	fn make_constant(&mut self, value: impl Into<Value>) -> u8 {
		let constant = self.current_chunk_mut().add_constant(value);

		match constant.try_into() {
			Ok(constant) => constant,
			Err(_) => {
				self.parser.error("Too many constants in one chunk.");
				0
			}
		}
	}

	fn emit_constant(&mut self, value: impl Into<Value>) {
		let constant = self.make_constant(value);
		self.emit_bytes([Opcode::Constant as u8, constant]);
	}

	fn emit_bytes<const N: usize>(&mut self, bytes: [u8; N]) {
		let line = self.parser.previous().line();
		for byte in bytes {
			self.current_chunk_mut().write(byte, line);
		}
	}

	#[inline]
	pub fn into_chunk(self) -> Chunk {
		self.compiling_chunk
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

type ParseFn<'a> = fn(&mut Compilation<'a>, can_assign: bool);

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
        Ty::LeftParen    => (Some(Compilation::grouping), None,                      Precedence::None),
        Ty::RightParen   => (None,                        None,                      Precedence::None),
        Ty::LeftBrace    => (None,                        None,                      Precedence::None),
        Ty::RightBrace   => (None,                        None,                      Precedence::None),
        Ty::Comma        => (None,                        None,                      Precedence::None),
        Ty::Dot          => (None,                        None,                      Precedence::None),
        Ty::Minus        => (Some(Compilation::unary),    Some(Compilation::binary), Precedence::Term),
        Ty::Plus         => (None,                        Some(Compilation::binary), Precedence::Term),
        Ty::Semicolon    => (None,                        None,                      Precedence::None),
        Ty::Slash        => (None,                        Some(Compilation::binary), Precedence::Factor),
        Ty::Star         => (None,                        Some(Compilation::binary), Precedence::Factor),
        Ty::Bang         => (Some(Compilation::unary),    None,                      Precedence::None),
        Ty::BangEqual    => (None,                        Some(Compilation::binary), Precedence::Equality),
        Ty::Equal        => (None,                        None,                      Precedence::None),
        Ty::EqualEqual   => (None,                        Some(Compilation::binary), Precedence::Equality),
        Ty::Greater      => (None,                        Some(Compilation::binary), Precedence::Comparison),
        Ty::GreaterEqual => (None,                        Some(Compilation::binary), Precedence::Comparison),
        Ty::Less         => (None,                        Some(Compilation::binary), Precedence::Comparison),
        Ty::LessEqual    => (None,                        Some(Compilation::binary), Precedence::Comparison),
        Ty::Identifier   => (Some(Compilation::variable), None,                      Precedence::None),
        Ty::String       => (Some(Compilation::string),   None,                      Precedence::None),
        Ty::Number       => (Some(Compilation::number),   None,                      Precedence::None),
        Ty::And          => (None,                        None,                      Precedence::None),
        Ty::Class        => (None,                        None,                      Precedence::None),
        Ty::Else         => (None,                        None,                      Precedence::None),
        Ty::False        => (Some(Compilation::literal),  None,                      Precedence::None),
        Ty::For          => (None,                        None,                      Precedence::None),
        Ty::Fun          => (None,                        None,                      Precedence::None),
        Ty::If           => (None,                        None,                      Precedence::None),
        Ty::Nil          => (Some(Compilation::literal),  None,                      Precedence::None),
        Ty::Or           => (None,                        None,                      Precedence::None),
        Ty::Print        => (None,                        None,                      Precedence::None),
        Ty::Return       => (None,                        None,                      Precedence::None),
        Ty::Super        => (None,                        None,                      Precedence::None),
        Ty::This         => (None,                        None,                      Precedence::None),
        Ty::True         => (Some(Compilation::literal),  None,                      Precedence::None),
        Ty::Var          => (None,                        None,                      Precedence::None),
        Ty::While        => (None,                        None,                      Precedence::None),
        Ty::Error        => (None,                        None,                      Precedence::None),
        Ty::Eof          => (None,                        None,                      Precedence::None),
    };
	ParseRule::new(prefix, infix, precedence)
}
