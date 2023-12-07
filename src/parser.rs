use crate::ast::*;
use peg;

peg::parser! {
    pub grammar expr_parser() for str {
        rule ws() = quiet!{[' ' | '\n' | '\t']*}

        rule number() -> Expr
            = ws() n:$(['0'..='9']+) ws() {? n.parse().map(|n| Expr::Number(n)).or(Err("number"))}

        rule identifier() -> Variable
            = ws() s:$(['a'..='z' | 'A'..='Z']+) ws() { Variable { name: s.to_owned(), id: None } }

        pub rule expr() -> Expr = precedence! {
            x:(@) "+" y:@ { Expr::Add(Box::new(x), Box::new(y)) }
            x:(@) "-" y:@ { Expr::Sub(Box::new(x), Box::new(y)) }
            --
            x:(@) "*" y:@ { Expr::Mul(Box::new(x), Box::new(y)) }
            x:(@) "/" y:@ { Expr::Div(Box::new(x), Box::new(y)) }
            --
            x:(@) ws() y:@ { Expr::App(Box::new(x), Box::new(y)) }
            --
            ws() "\\" v:identifier() "." e:expr() { Expr::Abs(v, Box::new(e)) }
            n:number() { n }
            name:identifier() { Expr::Var(name) }
            ws() "(" e:expr() ")" ws() { e }
        }

    }
}
