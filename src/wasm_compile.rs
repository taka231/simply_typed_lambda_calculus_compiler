use std::collections::{HashMap, HashSet};

use crate::{
    anf::{ANFs, HoistedANFs, Value, ANF},
    ast::{Operator, Variable},
};

pub struct WasmCompiler {
    pub program: String,
    pub fun_table: HashMap<Variable, u32>,
}

impl WasmCompiler {
    pub fn new() -> Self {
        Self {
            program: String::new(),
            fun_table: HashMap::new(),
        }
    }

    fn append_line(&mut self, line: &str) {
        self.program.push_str(line);
        self.program.push('\n');
    }

    fn append(&mut self, str: &str) {
        self.program.push_str(str);
    }

    pub fn compile(&mut self, hoisted_anfs: HoistedANFs) {
        self.append_line("(module");
        self.append_line("(memory 1)");
        self.append_line("(global $stack_pointer (mut i32) (i32.const 0))");
        self.generate_fun_table(&hoisted_anfs);
        self.append_line("(type $t (func (param i32 i32) (result i32)))");
        // start function definition
        for (fun_name, args, body) in hoisted_anfs.fun_defs {
            self.compile_fun(&fun_name.to_string(), args, &body);
        }
        self.compile_fun("_start", Vec::new(), &hoisted_anfs.main);
        // end function definition
        self.append_line("(export \"_start\" (func $_start))");
        self.append_line(")");
    }

    fn generate_fun_table(&mut self, hoisted_anfs: &HoistedANFs) {
        let fun_count = hoisted_anfs.fun_defs.len();
        self.append_line(&format!("(table {fun_count} funcref)"));
        let mut fun_names = String::new();
        for (i, (name, _, _)) in hoisted_anfs.fun_defs.iter().enumerate() {
            fun_names.push_str(&format!(" ${name}"));
            self.fun_table.insert(name.clone(), i as u32);
        }
        self.append_line(&format!("(elem (i32.const 0){fun_names})"));
    }

    fn compile_fun(&mut self, fun_name: &str, args: Vec<Variable>, body: &ANFs) {
        self.append(&format!("(func ${fun_name} "));
        for arg in &args {
            self.append(&format!("(param ${arg} i32) "));
        }
        self.append_line("(result i32)");
        let mut bound_vars = HashSet::new();
        for arg in &args {
            bound_vars.insert(arg.id);
        }
        let mut local_vars: HashSet<&Variable> = HashSet::new();
        for anf in &body.anfs {
            match anf {
                ANF::Fun(var, _, _) => {
                    local_vars.insert(var);
                }
                ANF::App(var, _, _) => {
                    local_vars.insert(var);
                }
                ANF::BOp(var, _, _, _) => {
                    local_vars.insert(var);
                }
                ANF::Tuple(var, _) => {
                    local_vars.insert(var);
                }
                ANF::Project(var, _, _) => {
                    local_vars.insert(var);
                }
            }
        }
        self.append_line(&local_vars.iter().fold(String::new(), |acc, var| {
            format!("(local ${var} i32) {acc}")
        }));
        for anf in &body.anfs {
            self.compile_anf(anf);
        }
        self.compile_value(&body.value.clone().unwrap());
        self.append_line(")");
    }

    fn compile_anf(&mut self, anf: &ANF) {
        match anf {
            ANF::Fun(_, _, _) => {
                unreachable!("hoisted anf should not have internal function definition")
            }
            ANF::App(var, func, args) => {
                for arg in args {
                    self.compile_value(arg);
                }
                self.append_line(&format!("(call_indirect (type $t) (local.get ${func}))"));
                self.append_line(&format!("local.set ${var}"));
            }
            ANF::BOp(var, op, v1, v2) => {
                self.compile_value(v1);
                self.compile_value(v2);
                match op {
                    Operator::Add => self.append_line("i32.add"),
                    Operator::Sub => self.append_line("i32.sub"),
                    Operator::Mul => self.append_line("i32.mul"),
                    Operator::Div => self.append_line("i32.div_s"),
                }
                self.append_line(&format!("local.set ${var}"));
            }
            ANF::Tuple(var, tuple) => {
                for (i, v) in tuple.iter().enumerate() {
                    self.append_line("global.get $stack_pointer");
                    self.compile_value(v);
                    self.append_line(&format!("i32.store offset={}", i * 4));
                }
                self.append_line("global.get $stack_pointer");
                self.append_line(&format!("local.set ${var}"));
                self.append_line("global.get $stack_pointer");
                self.append_line(&format!("i32.const {}", tuple.len() * 4));
                self.append_line("i32.add");
                self.append_line("global.set $stack_pointer");
            }
            ANF::Project(var, tuple, index) => {
                self.append_line(&format!("local.get ${tuple}"));
                self.append_line(&format!("i32.load offset={}", index * 4));
                self.append_line(&format!("local.set ${var}"));
            }
        }
    }

    fn compile_value(&mut self, value: &Value) {
        match value {
            Value::Number(n) => self.append_line(&format!("i32.const {n}")),
            Value::Var(var) => self.append_line(&format!("local.get ${var}")),
            Value::Global(var) => {
                let index = self.fun_table.get(var).unwrap();
                self.append_line(&format!("i32.const {index}"));
            }
        }
    }
}
