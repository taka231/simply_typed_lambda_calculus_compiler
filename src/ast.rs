#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    Var(Variable),
    Abs(Variable, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Variable {
    pub name: String,
    pub id: Option<usize>,
}
