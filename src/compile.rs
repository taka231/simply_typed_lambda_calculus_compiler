use std::collections::HashMap;

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{FunctionType, IntType, PointerType},
    values::IntValue,
    AddressSpace,
};

use crate::{
    anf::{HoistedANFs, Value, ANF},
    ast::Operator,
};

#[derive(Debug)]
pub struct LLVMCompiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub module: &'a Module<'ctx>,
    pub builder: &'a Builder<'ctx>,
    i64_type: IntType<'ctx>,
    fn_type: FunctionType<'ctx>,
    fn_ptr_type: PointerType<'ctx>,
}

impl<'a, 'ctx> LLVMCompiler<'a, 'ctx> {
    pub fn new(
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        module: &'a Module<'ctx>,
    ) -> Self {
        let i64_type = context.i64_type();
        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let fn_ptr_type = fn_type.ptr_type(AddressSpace::from(0));
        let malloc_type = i64_type
            .ptr_type(AddressSpace::from(0))
            .fn_type(&[i64_type.into()], false);
        module.add_function("malloc", malloc_type, None);
        Self {
            context,
            module,
            builder,
            i64_type,
            fn_type,
            fn_ptr_type,
        }
    }

    pub fn compile(&self, hoisted_anfs: HoistedANFs) {
        for (fun_name, args, body) in hoisted_anfs.fun_defs {
            let fun = self
                .module
                .add_function(&fun_name.to_string(), self.fn_type, None);
            let entry_basic_block = self.context.append_basic_block(fun, "entry");
            self.builder.position_at_end(entry_basic_block);

            let mut env: HashMap<String, IntValue> = HashMap::new();
            for (i, arg) in args.into_iter().enumerate() {
                let arg_ir = fun.get_nth_param(i as u32).unwrap();
                env.insert(arg.to_string(), arg_ir.into_int_value());
            }
            for anf in body.anfs {
                self.compile_anf(anf, &mut env);
            }
            let ret = self.compile_value(body.value.unwrap(), &mut env);
            self.builder.build_return(Some(&ret)).unwrap();
        }
        let main_fn_type = self.i64_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_fn_type, None);

        let entry_basic_block = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry_basic_block);

        let mut env: HashMap<String, IntValue> = HashMap::new();
        for anf in hoisted_anfs.main.anfs {
            self.compile_anf(anf, &mut env);
        }
        let ret = self.compile_value(hoisted_anfs.main.value.unwrap(), &mut env);
        self.builder.build_return(Some(&ret)).unwrap();
    }

    fn compile_anf(&self, anf: ANF, env: &mut HashMap<String, IntValue<'ctx>>) {
        match anf {
            ANF::Fun(_, _, _) => unreachable!(),
            ANF::App(var, fun_var, args) => {
                let fun = env.get(&fun_var.to_string()).unwrap();
                let fun = self
                    .builder
                    .build_int_to_ptr(*fun, self.fn_ptr_type, "ptr")
                    .unwrap();
                let args = args
                    .into_iter()
                    .map(|arg| self.compile_value(arg, env).into())
                    .collect::<Vec<_>>();
                let var_ir = self
                    .builder
                    .build_indirect_call(self.fn_type, fun, &args, &var.to_string())
                    .unwrap();
                env.insert(
                    var.to_string(),
                    var_ir.try_as_basic_value().unwrap_left().into_int_value(),
                );
            }
            ANF::BOp(var, op, val1, val2) => {
                let val1 = self.compile_value(val1, env);
                let val2 = self.compile_value(val2, env);
                let var_ir = match op {
                    Operator::Add => self.builder.build_int_add(val1, val2, &var.to_string()),
                    Operator::Sub => self.builder.build_int_sub(val1, val2, &var.to_string()),
                    Operator::Mul => self.builder.build_int_mul(val1, val2, &var.to_string()),
                    Operator::Div => {
                        self.builder
                            .build_int_signed_div(val1, val2, &var.to_string())
                    }
                }
                .unwrap();
                env.insert(var.to_string(), var_ir);
            }
            ANF::Tuple(var, tuple) => {
                let tuple = tuple
                    .into_iter()
                    .map(|val| self.compile_value(val, env))
                    .collect::<Vec<_>>();
                let malloc = self.module.get_function("malloc").unwrap();
                let tuple_ptr = self
                    .builder
                    .build_call(
                        malloc,
                        &[self
                            .i64_type
                            .const_int(8 * tuple.len() as u64, false)
                            .into()],
                        &var.to_string(),
                    )
                    .unwrap();
                for i in 0..tuple.len() {
                    let ptr = unsafe {
                        self.builder
                            .build_gep(
                                self.i64_type,
                                tuple_ptr
                                    .try_as_basic_value()
                                    .unwrap_left()
                                    .into_pointer_value(),
                                &[self.context.i32_type().const_int(i as u64, false).into()],
                                "ptr",
                            )
                            .unwrap()
                    };
                    self.builder.build_store(ptr, tuple[i]).unwrap();
                    let tuple_ptr = self
                        .builder
                        .build_ptr_to_int(
                            tuple_ptr
                                .try_as_basic_value()
                                .unwrap_left()
                                .into_pointer_value(),
                            self.i64_type,
                            "ptr",
                        )
                        .unwrap();
                    env.insert(var.to_string(), tuple_ptr);
                }
            }
            ANF::Project(var, tuple, index) => {
                let tuple = env.get(&tuple.to_string()).unwrap().clone();
                let tuple_ptr = self
                    .builder
                    .build_int_to_ptr(tuple, self.i64_type.ptr_type(AddressSpace::from(0)), "ptr")
                    .unwrap();
                let ptr = unsafe {
                    self.builder
                        .build_gep(
                            self.i64_type,
                            tuple_ptr,
                            &[self
                                .context
                                .i32_type()
                                .const_int(index as u64, false)
                                .into()],
                            "ptr",
                        )
                        .unwrap()
                };
                let var_ir = self
                    .builder
                    .build_load(self.i64_type, ptr, &var.to_string())
                    .unwrap();
                env.insert(var.to_string(), var_ir.into_int_value());
            }
        }
    }

    fn compile_value<'b>(
        &self,
        value: Value,
        env: &'b HashMap<String, IntValue<'ctx>>,
    ) -> IntValue<'ctx>
    where
        'ctx: 'b,
    {
        match value {
            Value::Number(n) => self.i64_type.const_int(n as u64, true),
            Value::Var(var) => env.get(&var.to_string()).unwrap().clone(),
            Value::Global(var) => {
                let fun_ptr = self
                    .module
                    .get_function(&var.to_string())
                    .unwrap()
                    .as_global_value()
                    .as_pointer_value();
                self.builder
                    .build_ptr_to_int(fun_ptr, self.i64_type, "ptr")
                    .unwrap()
            }
        }
    }
}
