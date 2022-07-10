use crate::expr::Expr;

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
    str.push_str("(");
    str.push_str(name);
    for expr in exprs {
        str.push_str(" ");
        str.push_str(&ast_to_string(expr));
    }
    str.push_str(")");
    str
}
