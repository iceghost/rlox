use std::mem::MaybeUninit;

use crate::{
    chunk::{Chunk, Opcode},
    debug,
    scanner::{
        token::{self, Token, Ty},
        Scanner,
    },
    value::Value,
};

pub struct Parser<'a> {
    current: MaybeUninit<Token<'a>>,
    previous: MaybeUninit<Token<'a>>,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Default for Parser<'a> {
    fn default() -> Self {
        let current = MaybeUninit::uninit();
        let previous = MaybeUninit::uninit();
        let had_error = false;
        let panic_mode = false;
        Self {
            current,
            previous,
            had_error,
            panic_mode,
        }
    }
}

impl<'a> Parser<'a> {
    #[inline]
    fn previous(&self) -> Token<'a> {
        unsafe { self.previous.assume_init() }
    }

    #[inline]
    fn current(&self) -> Token<'a> {
        unsafe { self.current.assume_init() }
    }

    fn advance(&mut self, scanner: &mut Scanner<'a>) {
        let next_token = loop {
            let token = scanner.scan_token();
            if token.ty() != Ty::Error {
                break token;
            };
            self.error_at_current(token.lexeme());
        };
        // Token implemented Copy so we don't need this
        // unsafe { self.previous.assume_init_drop() };
        self.previous = std::mem::replace(&mut self.current, MaybeUninit::new(next_token));
    }

    fn consume(&mut self, scanner: &mut Scanner<'a>, ty: Ty, message: &str) {
        if self.current().ty() != ty {
            self.error_at_current(message);
            return;
        }

        self.advance(scanner);
    }

    #[inline]
    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current(), message);
    }

    #[inline]
    fn error(&mut self, message: &str) {
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
}

#[derive(Default)]
pub struct Compiler<'a> {
    parser: Parser<'a>,
    compiling_chunk: Chunk,
}

impl<'a> Compiler<'a> {
    pub fn compile(&mut self, source: &'a str) -> bool {
        let mut scanner = Scanner::new(source);
        self.parser.advance(&mut scanner);
        self.expression(&mut scanner);
        self.end();
        self.parser
            .consume(&mut scanner, Ty::Eof, "Expect end of expression.");

        !self.parser.had_error
    }

    fn end(&mut self) {
        self.emit_bytes([Opcode::Return as u8]);
        if !self.parser.had_error {
            debug::disassemble_chunk(self.current_chunk_mut(), "code");
        }
    }

    fn binary(&mut self, scanner: &mut Scanner<'a>) {
        let operator = self.parser.previous().ty();
        let rule = get_rule(operator);
        self.parse_precedence(scanner, rule.precedence.successor());

        match operator {
            Ty::Plus => self.emit_bytes([Opcode::Add as u8]),
            Ty::Minus => self.emit_bytes([Opcode::Subtract as u8]),
            Ty::Star => self.emit_bytes([Opcode::Multiply as u8]),
            Ty::Slash => self.emit_bytes([Opcode::Divide as u8]),
            _ => unreachable!(),
        }
    }

    fn grouping(&mut self, scanner: &mut Scanner<'a>) {
        self.expression(scanner);
        self.parser
            .consume(scanner, Ty::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self, scanner: &mut Scanner<'a>) {
        let operator = self.parser.previous().ty();

        self.parse_precedence(scanner, Precedence::Unary);

        match operator {
            Ty::Minus => self.emit_bytes([Opcode::Negate as u8]),
            _ => unreachable!(),
        }
    }

    fn expression(&mut self, scanner: &mut Scanner<'a>) {
        self.parse_precedence(scanner, Precedence::Assignment);
    }

    fn parse_precedence(&mut self, scanner: &mut Scanner<'a>, prec: Precedence) {
        self.parser.advance(scanner);
        let prefix_rule = get_rule(self.parser.previous().ty()).prefix;
        let prefix_rule = if let Some(prefix_rule) = prefix_rule {
            prefix_rule
        } else {
            self.parser.error("Expect expression");
            return;
        };

        prefix_rule(self, scanner);

        while prec <= get_rule(self.parser.current().ty()).precedence {
            self.parser.advance(scanner);
            let infix_rule = get_rule(self.parser.previous().ty()).infix.unwrap();
            infix_rule(self, scanner);
        }
    }

    fn number(&mut self, _: &mut Scanner<'a>) {
        let value = self.parser.previous().lexeme().parse::<f64>().unwrap();
        self.emit_constant(value);
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

type ParseFn<'a> = for<'r, 's> fn(&'r mut Compiler<'a>, &'s mut Scanner<'a>);

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
    let (prefix, infix, precedence): (Option<ParseFn>, Option<ParseFn>, Precedence) = match operator
    {
        Ty::LeftParen => (Some(Compiler::grouping), None, Precedence::None),
        Ty::RightParen => (None, None, Precedence::None),
        Ty::LeftBrace => (None, None, Precedence::None),
        Ty::RightBrace => (None, None, Precedence::None),
        Ty::Comma => (None, None, Precedence::None),
        Ty::Dot => (None, None, Precedence::None),
        Ty::Minus => (
            Some(Compiler::unary),
            Some(Compiler::binary),
            Precedence::Term,
        ),
        Ty::Plus => (None, Some(Compiler::binary), Precedence::Term),
        Ty::Semicolon => (None, None, Precedence::None),
        Ty::Slash => (None, Some(Compiler::binary), Precedence::Factor),
        Ty::Star => (None, Some(Compiler::binary), Precedence::Factor),
        Ty::Bang => (None, None, Precedence::None),
        Ty::BangEqual => (None, None, Precedence::None),
        Ty::Equal => (None, None, Precedence::None),
        Ty::EqualEqual => (None, None, Precedence::None),
        Ty::Greater => (None, None, Precedence::None),
        Ty::GreaterEqual => (None, None, Precedence::None),
        Ty::Less => (None, None, Precedence::None),
        Ty::LessEqual => (None, None, Precedence::None),
        Ty::Identifier => (None, None, Precedence::None),
        Ty::String => (None, None, Precedence::None),
        Ty::Number => (Some(Compiler::number), None, Precedence::None),
        Ty::And => (None, None, Precedence::None),
        Ty::Class => (None, None, Precedence::None),
        Ty::Else => (None, None, Precedence::None),
        Ty::False => (None, None, Precedence::None),
        Ty::For => (None, None, Precedence::None),
        Ty::Fun => (None, None, Precedence::None),
        Ty::If => (None, None, Precedence::None),
        Ty::Nil => (None, None, Precedence::None),
        Ty::Or => (None, None, Precedence::None),
        Ty::Print => (None, None, Precedence::None),
        Ty::Return => (None, None, Precedence::None),
        Ty::Super => (None, None, Precedence::None),
        Ty::This => (None, None, Precedence::None),
        Ty::True => (None, None, Precedence::None),
        Ty::Var => (None, None, Precedence::None),
        Ty::While => (None, None, Precedence::None),
        Ty::Error => (None, None, Precedence::None),
        Ty::Eof => (None, None, Precedence::None),
    };
    ParseRule::new(prefix, infix, precedence)
}
