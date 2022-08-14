use crate::{
	chunk::{Chunk, Opcode},
	debug,
	scanner::token::Ty,
	value::Value,
	vm::VM,
};

mod parser;

use self::parser::Parser;

#[derive(Default)]
struct Compiler<'a> {
	locals: Vec<Local<'a>>,
	scope_depth: u8,
}

struct Local<'a> {
	name: &'a str,
	depth: Option<u8>,
}

pub struct Compilation<'a> {
	parser: Parser<'a>,
	current: Compiler<'a>,
	compiling_chunk: Chunk,
	vm: &'a mut VM,
}

impl<'a> Compilation<'a> {
	pub fn new(vm: &'a mut VM, source: &'a str) -> Self {
		let parser = Parser::new(source);
		let compiling_chunk = Chunk::default();
		let current = Compiler::default();
		Self {
			current,
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
		self.declare_variable();
		if self.current.scope_depth > 0 {
			0
		} else {
			self.identifier_constant(self.parser.previous().lexeme())
		}
	}

	fn declare_variable(&mut self) {
		if self.current.scope_depth == 0 {
			return;
		}
		let name = self.parser.previous().lexeme();
		for local in self.current.locals.iter().rev() {
			if local.depth.is_some() && local.depth < Some(self.current.scope_depth) {
				break;
			}
			if name == local.name {
				self.parser
					.error("Already a variable with this name in this scope.");
			}
		}
		self.add_local(name);
	}

	fn add_local(&mut self, name: &'a str) {
		if self.current.locals.len() == u8::MAX as usize {
			self.parser.error("Too many local variables in function.");
			return;
		}
		self.current.locals.push(Local { name, depth: None });
	}

	fn identifier_constant(&mut self, name: &str) -> u8 {
		let obj = self.vm.allocate_string(name.to_owned());
		self.make_constant(obj)
	}

	fn define_variable(&mut self, global: u8) {
		if self.current.scope_depth > 0 {
			self.mark_initialized();
			return;
		}
		self.emit_bytes([Opcode::DefineGlobal as u8, global]);
	}

	fn and(&mut self, _: bool) {
		let end_jump = self.emit_jump(Opcode::JumpIfFalse);
		self.emit_bytes([Opcode::Pop as u8]);
		self.parse_precedence(Precedence::And);
		self.patch_jump(end_jump);
	}

	fn or(&mut self, _: bool) {
		let else_jump = self.emit_jump(Opcode::JumpIfFalse);
		let end_jump = self.emit_jump(Opcode::Jump);
		self.patch_jump(else_jump);
		self.emit_bytes([Opcode::Pop as u8]);
		self.parse_precedence(Precedence::Or);
		self.patch_jump(end_jump);
	}

	fn mark_initialized(&mut self) {
		let last = self.current.locals.last_mut().unwrap();
		last.depth = Some(self.current.scope_depth);
	}

	fn statement(&mut self) {
		if self.parser.matches(Ty::Print) {
			self.print_statement();
		} else if self.parser.matches(Ty::If) {
			self.if_statement();
		} else if self.parser.matches(Ty::While) {
			self.while_statement();
		} else if self.parser.matches(Ty::For) {
			self.for_statement();
		} else if self.parser.matches(Ty::LeftBrace) {
			self.begin_scope();
			self.block();
			self.end_scope();
		} else {
			self.expression_statement();
		}
	}

	fn if_statement(&mut self) {
		self.parser.consume(Ty::LeftParen, "Expect '(' after 'if'.");
		self.expression();
		self.parser
			.consume(Ty::RightParen, "Expect ')' after 'if'.");

		let then_jump = self.emit_jump(Opcode::JumpIfFalse);
		self.emit_bytes([Opcode::Pop as u8]);
		self.statement();
		let else_jump = self.emit_jump(Opcode::Jump);
		self.patch_jump(then_jump);
		self.emit_bytes([Opcode::Pop as u8]);

		if self.parser.matches(Ty::Else) {
			self.statement();
		}
		self.patch_jump(else_jump);
	}

	fn for_statement(&mut self) {
		self.begin_scope();
		self.parser
			.consume(Ty::LeftParen, "Expect '(' after 'for'.");

		if self.parser.matches(Ty::Semicolon) {
		} else if self.parser.matches(Ty::Var) {
			self.var_declaration();
		} else {
			self.expression_statement();
		}

		let mut loop_start = self.current_chunk().len();
		let exit_jump = if !self.parser.matches(Ty::Semicolon) {
			self.expression();
			self.parser
				.consume(Ty::Semicolon, "Expect ';' after loop condition.");
			let exit_jump = self.emit_jump(Opcode::JumpIfFalse);
			self.emit_bytes([Opcode::Pop as u8]);
			Some(exit_jump)
		} else {
			None
		};

		if !self.parser.matches(Ty::RightParen) {
			let body_jump = self.emit_jump(Opcode::Jump);
			let increment_start = self.current_chunk().len();
			self.expression();
			self.emit_bytes([Opcode::Pop as u8]);
			self.parser
				.consume(Ty::RightParen, "Expect ')' after for clauses.");

			self.emit_loop(loop_start);
			loop_start = increment_start;
			self.patch_jump(body_jump);
		}

		self.statement();
		self.emit_loop(loop_start);

		if let Some(exit_jump) = exit_jump {
			self.patch_jump(exit_jump);
			self.emit_bytes([Opcode::Pop as u8]);
		}

		self.end_scope();
	}

	fn block(&mut self) {
		while !self.parser.check(Ty::RightBrace) && !self.parser.check(Ty::Eof) {
			self.declaration();
		}
		self.parser
			.consume(Ty::RightBrace, "Expect '}' after block.");
	}

	fn begin_scope(&mut self) {
		self.current.scope_depth += 1;
	}

	fn end_scope(&mut self) {
		self.current.scope_depth -= 1;

		while let Some(local) = self.current.locals.last() {
			if local.depth > Some(self.current.scope_depth) {
				self.emit_bytes([Opcode::Pop as u8]);
				self.current.locals.pop();
			} else {
				break;
			}
		}
	}

	fn print_statement(&mut self) {
		self.expression();
		self.parser
			.consume(Ty::Semicolon, "Expect ';' after value.");
		self.emit_bytes([Opcode::Print as u8])
	}

	fn while_statement(&mut self) {
		let loop_start = self.current_chunk().len();

		self.parser
			.consume(Ty::LeftParen, "Expect '(' after 'while'.");
		self.expression();
		self.parser
			.consume(Ty::RightParen, "Expect ')' after condition.");

		let exit_jump = self.emit_jump(Opcode::JumpIfFalse);
		self.emit_bytes([Opcode::Pop as u8]);
		self.statement();
		self.emit_loop(loop_start);

		self.patch_jump(exit_jump);
		self.emit_bytes([Opcode::Pop as u8]);
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

	fn named_variable(&mut self, name: &'a str, can_assign: bool) {
		// let current = &self.current;
		let (arg, get_op, set_op) = match self.resolve_local(name) {
			None => (
				self.identifier_constant(name),
				Opcode::GetGlobal,
				Opcode::SetGlobal,
			),
			Some(i) => (i as u8, Opcode::GetLocal, Opcode::SetLocal),
		};
		if can_assign && self.parser.matches(Ty::Equal) {
			self.expression();
			self.emit_bytes([set_op as u8, arg]);
		} else {
			self.emit_bytes([get_op as u8, arg]);
		}
	}

	fn resolve_local(&mut self, name: &'a str) -> Option<u8> {
		for (i, local) in self.current.locals.iter().enumerate().rev() {
			if name == local.name {
				if local.depth.is_none() {
					self.parser
						.error("Can't read local variable in its own initializer.");
				}
				return Some(i as u8);
			}
		}
		None
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

	fn emit_jump(&mut self, instruction: Opcode) -> usize {
		self.emit_bytes([instruction as u8, 0xff, 0xff]);
		self.current_chunk_mut().len() - 2
	}

	fn patch_jump(&mut self, offset: usize) {
		let jump = self.current_chunk().len() as isize - offset as isize - 2;
		if jump > u16::MAX as isize {
			self.parser.error("Too much code to jump over.");
		}
		let code = self.current_chunk_mut().code_mut();
		code[offset] = (jump >> 8) as u8;
		code[offset + 1] = jump as u8;
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

	fn emit_loop(&mut self, loop_start: usize) {
		self.emit_bytes([Opcode::Loop as u8]);

		let offset = self.current_chunk().len() - loop_start + 2;
		if offset > u16::MAX as usize {
			self.parser.error("Loop body too large.");
		}
		self.emit_bytes((offset as u16).to_be_bytes());
	}

	#[inline]
	pub fn into_chunk(self) -> Chunk {
		self.compiling_chunk
	}

	#[inline]
	fn current_chunk_mut(&mut self) -> &mut Chunk {
		&mut self.compiling_chunk
	}

	#[inline]
	fn current_chunk(&mut self) -> &Chunk {
		&self.compiling_chunk
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
        Ty::And          => (None,                        Some(Compilation::and),    Precedence::And),
        Ty::Class        => (None,                        None,                      Precedence::None),
        Ty::Else         => (None,                        None,                      Precedence::None),
        Ty::False        => (Some(Compilation::literal),  None,                      Precedence::None),
        Ty::For          => (None,                        None,                      Precedence::None),
        Ty::Fun          => (None,                        None,                      Precedence::None),
        Ty::If           => (None,                        None,                      Precedence::None),
        Ty::Nil          => (Some(Compilation::literal),  None,                      Precedence::None),
        Ty::Or           => (None,                        Some(Compilation::or),     Precedence::Or),
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
