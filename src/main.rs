use SimplyTypedLambdaCalculusCompiler::{
    alpha::AlphaConvEnv,
    anf::{ANFConverter, ANFs},
    parser::expr_parser,
    typeinfer::TypeInfer,
};

fn main() {
    let ast = expr_parser::expr(r#"(\f. \x. f x) ((\x. \y. x + y) 2) 3"#).unwrap();
    let alpha_conv_env = AlphaConvEnv::new();
    let ast = alpha_conv_env.alpha_conversion(ast).unwrap();
    let mut typeinfer = TypeInfer::new(alpha_conv_env.id());
    let ty = typeinfer.type_infer(&ast).unwrap();
    println!("{:?}", ty.simplify());
    let mut anfconverter = ANFConverter::new(alpha_conv_env.id());
    let mut anfs = ANFs {
        anfs: Vec::new(),
        value: None,
        level: 0,
    };
    anfconverter.convert(ast, &mut anfs);
    let anfs = anfconverter.closure_conversion(anfs);
    println!("{}", anfs);
}
