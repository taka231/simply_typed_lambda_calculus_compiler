use SimplyTypedLambdaCalculusCompiler::parser::expr_parser;

fn main() {
    println!("{:?}", expr_parser::expr(r#"1 - 2 * 3 - 4"#));
}
