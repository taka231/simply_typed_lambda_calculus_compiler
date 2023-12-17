use crate::ast::*;
use peg;

peg::parser! {
    pub grammar expr_parser() for str {
        rule _ = quiet!{[' ' | '\n' | '\t']*}

        rule number() -> Expr
            = _ n:$(['0'..='9']+) _ {? n.parse().map(|n| Expr::Number(n)).or(Err("number"))}

        rule identifier() -> Variable
            = _ s:$(['a'..='z' | 'A'..='Z']+) _ { Variable { name: s.to_owned(), id: None } }

        pub rule expr() -> Expr = precedence! {
            x:(@) "+" y:@ { Expr::BOp(Operator::Add, Box::new(x), Box::new(y)) }
            x:(@) "-" y:@ { Expr::BOp(Operator::Sub, Box::new(x), Box::new(y)) }
            --
            x:(@) "*" y:@ { Expr::BOp(Operator::Mul, Box::new(x), Box::new(y)) }
            x:(@) "/" y:@ { Expr::BOp(Operator::Div, Box::new(x), Box::new(y)) }
            --
            x:(@) _ y:@ { Expr::App(Box::new(x), Box::new(y)) }
            --
            _ "\\" v:identifier() "." e:expr() { Expr::Abs(v, Box::new(e)) }
            n:number() { n }
            name:identifier() { Expr::Var(name) }
            _ "(" e:expr() ")" _ { e }
        }

    }
}
