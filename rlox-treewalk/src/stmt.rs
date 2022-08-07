use std::rc::Rc;

use crate::{expr::Expr, token::Token};

pub struct StmtFunction {
	pub name: Token,
	pub params: Vec<Token>,
	pub body: Vec<Stmt>,
}

pub enum Stmt {
	Expression(Expr),
	Print(Expr),
	Var {
		name: Token,
		initializer: Option<Expr>,
	},
	If {
		condition: Expr,
		then_branch: Box<Stmt>,
		else_branch: Option<Box<Stmt>>,
	},
	While {
		condition: Expr,
		body: Box<Stmt>,
	},
	Function(Rc<StmtFunction>),
	Return {
		keyword: Token,
		value: Expr,
	},
	Block(Vec<Stmt>),
}
