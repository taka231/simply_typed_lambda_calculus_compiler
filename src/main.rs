use SimplyTypedLambdaCalculusCompiler::parser::expr_parser;

fn main() {
    println!(
        "{:?}",
        expr_parser::expr(r#"(\x. \y. x + y) (1 + 1) (1 + 1 * 2)"#)
    );
}
