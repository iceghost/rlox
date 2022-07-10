use crate::expr::Expr;

#[allow(unused)]
pub fn ast_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Binary {
            left,
            operator,
            right,
        } => parenthesize(&operator.lexeme, &[left, right]),
        Expr::Grouping(expr) => parenthesize("group", &[expr]),
        Expr::Literal(lit) => format!("{lit}"),
        Expr::Unary { operator, right } => parenthesize(&operator.lexeme, &[right]),
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
