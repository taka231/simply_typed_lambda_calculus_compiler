use inkwell::context::Context;
use simply_typed_lambda_calculus_compiler::{
    alpha::AlphaConvEnv,
    anf::{ANFConverter, ANFs, HoistedANFs},
    compile::LLVMCompiler,
    parser::expr_parser,
    typeinfer::TypeInfer,
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "simply_typed_lambda_calculus_compiler")]
struct Opt {
    /// show result type
    #[structopt(short, long = "type")]
    type_: bool,

    /// show a-normal form
    #[structopt(short, long)]
    anf: bool,

    /// show closure converted a-normal form
    #[structopt(short, long)]
    closure: bool,

    /// show hoisted a-normal form
    #[structopt(short, long)]
    hoist: bool,

    /// show llvm ir
    #[structopt(short, long)]
    llvm: bool,

    program: String,
}

fn main() {
    let Opt {
        type_,
        anf,
        closure,
        hoist,
        llvm,
        program,
    } = Opt::from_args();
    let ast = expr_parser::expr(&program).unwrap();
    let alpha_conv_env = AlphaConvEnv::new();
    let ast = alpha_conv_env.alpha_conversion(ast).unwrap();
    let mut typeinfer = TypeInfer::new(alpha_conv_env.id());
    let ty = typeinfer.type_infer(&ast).unwrap();
    if type_ {
        println!("Type: {:?}\n", ty.simplify());
    }
    let mut anfconverter = ANFConverter::new(alpha_conv_env.id());
    let mut anfs = ANFs {
        anfs: Vec::new(),
        value: None,
        level: 0,
    };
    anfconverter.convert(ast, &mut anfs);
    if anf {
        println!("ANF:{}\n", &anfs);
    }
    let anfs = anfconverter.closure_conversion(anfs);
    if closure {
        println!("closure converted ANF:{}\n", &anfs);
    }
    let mut hoisted_anfs = HoistedANFs {
        fun_defs: Vec::new(),
        main: ANFs {
            anfs: Vec::new(),
            value: None,
            level: 1,
        },
    };
    anfconverter.hoisting(anfs, &mut hoisted_anfs);
    if hoist {
        println!("hoisted ANF:\n{}\n", &hoisted_anfs);
    }
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("main");
    let llvm_compiler = LLVMCompiler::new(&context, &builder, &module);
    llvm_compiler.compile(hoisted_anfs);
    if llvm {
        llvm_compiler.module.print_to_stderr();
    }
    let execution_engine = llvm_compiler
        .module
        .create_jit_execution_engine(inkwell::OptimizationLevel::Aggressive)
        .unwrap();
    unsafe {
        let r = execution_engine
            .get_function::<unsafe extern "C" fn() -> i64>("main")
            .unwrap()
            .call();
        println!("{:?}", r);
    }
}
