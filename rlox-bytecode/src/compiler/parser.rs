use crate::scanner::token::Ty;
use crate::scanner::Scanner;

use crate::scanner::token::Token;

use std::mem::MaybeUninit;

pub struct Parser<'a> {
	scanner: Scanner<'a>,
	current: MaybeUninit<Token<'a>>,
	previous: MaybeUninit<Token<'a>>,
	had_error: bool,
	panic_mode: bool,
}

impl<'a> Parser<'a> {
	pub fn new(source: &'a str) -> Self {
		let current = MaybeUninit::uninit();
		let previous = MaybeUninit::uninit();
		let had_error = false;
		let panic_mode = false;
		let scanner = Scanner::new(source);
		let mut parser = Self {
			current,
			previous,
			had_error,
			panic_mode,
			scanner,
		};
		// prime the parser
		parser.advance();
		parser
	}

	#[inline]
	pub fn previous(&self) -> Token<'a> {
		unsafe { self.previous.assume_init() }
	}

	#[inline]
	pub fn current(&self) -> Token<'a> {
		unsafe { self.current.assume_init() }
	}

	pub fn synchronize(&mut self) {
		self.panic_mode = false;
		while self.current().ty() != Ty::Eof {
			if self.previous().ty() == Ty::Semicolon {
				return;
			}
			match self.current().ty() {
				Ty::Class
				| Ty::Fun
				| Ty::Var
				| Ty::For
				| Ty::If
				| Ty::While
				| Ty::Print
				| Ty::Return => {
					return;
				}
				_ => self.advance(),
			}
		}
	}

	pub fn advance(&mut self) {
		let next_token = loop {
			let token = self.scanner.scan_token();
			if token.ty() != Ty::Error {
				break token;
			};
			self.error_at_current(token.lexeme());
		};
		// Token implemented Copy so we don't need this
		// unsafe { self.previous.assume_init_drop() };
		self.previous = std::mem::replace(&mut self.current, MaybeUninit::new(next_token));
	}

	pub fn consume(&mut self, ty: Ty, message: &str) {
		if self.current().ty() != ty {
			self.error_at_current(message);
			return;
		}

		self.advance();
	}

	fn check(&self, ty: Ty) -> bool {
		self.current().ty() == ty
	}

	pub fn matches(&mut self, ty: Ty) -> bool {
		if !self.check(ty) {
			return false;
		}
		self.advance();
		true
	}

	#[inline]
	pub fn error_at_current(&mut self, message: &str) {
		self.error_at(self.current(), message);
	}

	#[inline]
	pub fn error(&mut self, message: &str) {
		self.error_at(self.previous(), message);
	}

	fn error_at(&mut self, token: Token, message: &str) {
		if self.panic_mode {
			return;
		}
		self.panic_mode = true;

		eprint!("[line {}] Error", token.line());

		if token.ty() == Ty::Eof {
			eprint!(" at end");
		} else if token.ty() == Ty::Error {
			// nothing
		} else {
			eprint!("at '{}'", token.lexeme());
		}

		eprintln!(": {message}");

		self.had_error = true;
	}

	pub fn had_error(&self) -> bool {
		self.had_error
	}

	#[allow(unused)]
	pub fn panic_mode(&self) -> bool {
		self.panic_mode
	}
}
