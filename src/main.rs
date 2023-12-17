use SimplyTypedLambdaCalculusCompiler::{
    alpha::AlphaConvEnv,
    anf::{ANFConverter, ANFs},
    parser::expr_parser,
    typeinfer::TypeInfer,
};

fn main() {
    let ast = expr_parser::expr(r#"(\f. \x. f x) ((\x. \y. x + y) 1) 1"#).unwrap();
    let alpha_conv_env = AlphaConvEnv::new();
    let ast = alpha_conv_env.alpha_conversion(ast).unwrap();
    let mut type_infer = TypeInfer::new(alpha_conv_env.id());
    let type_ = type_infer.type_infer(&ast).unwrap();
    let mut anfconverter = ANFConverter::new(type_infer.next_tvar, type_infer.env);
    let mut anfs = ANFs {
        anfs: Vec::new(),
        value: None,
    };
    anfconverter.convert(ast, &mut anfs);
    let anfs = anfconverter.closure_conversion(anfs);
    println!("{}", anfs);
}
