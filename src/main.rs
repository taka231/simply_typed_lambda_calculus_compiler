use SimplyTypedLambdaCalculusCompiler::{
    alpha::AlphaConvEnv,
    anf::{ANFConverter, ANFs},
    parser::expr_parser,
};

fn main() {
    let ast = expr_parser::expr(r#"(\x. \y. x + y) 2 3"#).unwrap();
    let alpha_conv_env = AlphaConvEnv::new();
    let ast = alpha_conv_env.alpha_conversion(ast).unwrap();
    let mut anfconverter = ANFConverter::new(alpha_conv_env.id());
    let mut anfs = ANFs {
        anfs: Vec::new(),
        value: None,
    };
    anfconverter.convert(ast, &mut anfs);
    let anfs = anfconverter.closure_conversion(anfs);
    println!("{}", anfs);
}
