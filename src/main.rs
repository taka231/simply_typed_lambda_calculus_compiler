use inkwell::context::Context;
use SimplyTypedLambdaCalculusCompiler::{
    alpha::AlphaConvEnv,
    anf::{ANFConverter, ANFs, HoistedANFs},
    compile::LLVMCompiler,
    parser::expr_parser,
    typeinfer::TypeInfer,
};

fn main() {
    let ast = expr_parser::expr(r#"(\f. \x. f x) ((\x. \y. x + y) 2) 3"#).unwrap();
    let alpha_conv_env = AlphaConvEnv::new();
    let ast = alpha_conv_env.alpha_conversion(ast).unwrap();
    let mut typeinfer = TypeInfer::new(alpha_conv_env.id());
    let ty = typeinfer.type_infer(&ast).unwrap();
    // println!("{:?}", ty.simplify());
    let mut anfconverter = ANFConverter::new(alpha_conv_env.id());
    let mut anfs = ANFs {
        anfs: Vec::new(),
        value: None,
        level: 0,
    };
    anfconverter.convert(ast, &mut anfs);
    let anfs = anfconverter.closure_conversion(anfs);
    let mut hoisted_anfs = HoistedANFs {
        fun_defs: Vec::new(),
        main: ANFs {
            anfs: Vec::new(),
            value: None,
            level: 1,
        },
    };
    anfconverter.hoisting(anfs, &mut hoisted_anfs);
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("main");
    let llvm_compiler = LLVMCompiler::new(&context, &builder, &module);
    llvm_compiler.compile(hoisted_anfs);
    llvm_compiler.module.print_to_stderr();
}
