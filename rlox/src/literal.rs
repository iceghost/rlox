pub enum Literal {
    Number(f64),
    String(String),
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Number(n) => n.fmt(f),
            Literal::String(s) => s.fmt(f),
        }
    }
}
