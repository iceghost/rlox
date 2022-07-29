use std::borrow::Cow;

use crate::expr::Expr;

#[allow(unused)]
pub fn ast_to_string(expr: &Expr) -> Cow<'_, str> {
    match expr {
        Expr::Binary {
            left,
            operator,
            right,
        } => parenthesize(&operator.lexeme, &[left, right]).into(),
        Expr::Grouping(expr) => parenthesize("group", &[expr]).into(),
        Expr::Literal(lit) => format!("{lit}").into(),
        Expr::Unary { operator, right } => parenthesize(&operator.lexeme, &[right]).into(),
        Expr::Variable(name) => (&name.lexeme).into(),
        _ => unimplemented!(),
    }
}

fn parenthesize(name: &str, exprs: &[&Expr]) -> String {
    let mut str = String::new();
    str.push('(');
    str.push_str(name);
    for expr in exprs {
        str.push(' ');
        str.push_str(&ast_to_string(expr));
    }
    str.push(')');
    str
}
