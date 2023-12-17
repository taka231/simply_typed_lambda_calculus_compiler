use core::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    Var(Variable),
    Abs(Variable, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Number(i64),
    BOp(Operator, Box<Expr>, Box<Expr>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Variable {
    pub name: String,
    pub id: Option<u32>,
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.id {
            Some(id) => write!(f, "{}_{}", self.name, id),
            None => write!(f, "{}", self.name),
        }
    }
}
