use core::fmt;
use std::collections::HashSet;

use crate::{
    ast::{Expr, Operator, Variable},
    typeinfer::Type,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    Number(i64),
    Var(Variable),
    Global(Variable),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Var(var) => write!(f, "{}", var),
            Value::Global(var) => write!(f, "@{}", var),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ANF {
    Fun(Variable, Vec<Variable>, ANFs),
    App(Variable, Variable, Vec<Value>),
    BOp(Variable, Operator, Value, Value),
    Tuple(Variable, Vec<Value>),
    Project(Variable, Variable, usize),
}

impl fmt::Display for ANF {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ANF::Fun(var, args, anfs) => {
                write!(f, "{}(", var)?;
                for i in 0..args.len() - 1 {
                    write!(f, "{}, ", args[i])?;
                }
                write!(f, "{}) = ", args[args.len() - 1])?;
                write!(f, "{}", anfs)?;
            }
            ANF::App(var, func, args) => {
                write!(f, "{} = {}(", var, func)?;
                for i in 0..args.len() - 1 {
                    write!(f, "{}, ", args[i])?;
                }
                write!(f, "{})", args[args.len() - 1])?;
            }
            ANF::BOp(var, op, val1, val2) => {
                write!(f, "{}_{} = ", var.name, var.id.unwrap())?;
                match op {
                    Operator::Add => write!(f, "{} + {}", val1, val2)?,
                    Operator::Sub => write!(f, "{} - {}", val1, val2)?,
                    Operator::Mul => write!(f, "{} * {}", val1, val2)?,
                    Operator::Div => write!(f, "{} / {}", val1, val2)?,
                }
            }
            ANF::Tuple(var, tuple) => {
                write!(f, "{} = (", var)?;
                for i in 0..tuple.len() - 1 {
                    write!(f, "{}, ", tuple[i])?;
                }
                write!(f, "{})", tuple[tuple.len() - 1])?;
            }
            ANF::Project(var, tuple, index) => {
                write!(f, "{}_{} = {}[{}]", var.name, var.id.unwrap(), tuple, index)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ANFs {
    pub anfs: Vec<ANF>,
    pub value: Option<Value>,
}

impl ANFs {
    fn free_vars(&self, bound_vars: &mut HashSet<u32>) -> Vec<Variable> {
        let mut free_vars = Vec::new();
        for anf in &self.anfs {
            match anf {
                ANF::Fun(var, args, body) => {
                    bound_vars.insert(var.id.unwrap());
                    for arg in args {
                        bound_vars.insert(arg.id.unwrap());
                    }
                    let mut body_free_vars = body.free_vars(bound_vars);
                    free_vars.append(&mut body_free_vars);
                }
                ANF::App(var1, var2, args) => {
                    bound_vars.insert(var1.id.unwrap());
                    if !bound_vars.contains(&var2.id.unwrap()) {
                        free_vars.push(var2.clone());
                    }
                    for arg in args {
                        match arg {
                            Value::Var(var) => {
                                if !bound_vars.contains(&var.id.unwrap()) {
                                    free_vars.push(var.clone());
                                }
                            }
                            _ => (),
                        }
                    }
                }
                ANF::BOp(var, _, val1, val2) => {
                    bound_vars.insert(var.id.unwrap());
                    match val1 {
                        Value::Var(var) => {
                            if !bound_vars.contains(&var.id.unwrap()) {
                                free_vars.push(var.clone());
                            }
                        }
                        _ => (),
                    }
                    match val2 {
                        Value::Var(var) => {
                            if !bound_vars.contains(&var.id.unwrap()) {
                                free_vars.push(var.clone());
                            }
                        }
                        _ => (),
                    }
                }
                ANF::Tuple(_, _) => unreachable!(),
                ANF::Project(_, _, _) => unreachable!(),
            }
        }
        if let Some(value) = &self.value {
            match value {
                Value::Var(var) => {
                    if !bound_vars.contains(&var.id.unwrap()) {
                        free_vars.push(var.clone());
                    }
                }
                _ => (),
            }
        }
        free_vars
    }
}

impl fmt::Display for ANFs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.anfs.len() != 0 {
            write!(f, "\n")?;
        }
        for anf in &self.anfs {
            write!(f, "let {} in\n", anf)?;
        }
        match &self.value {
            Some(val) => write!(f, "{}", val)?,
            None => write!(f, "return ()")?,
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ANFConverter {
    pub next_var: u32,
    pub type_env: Vec<Type>,
}

impl ANFConverter {
    pub fn new(next_var: u32, type_env: Vec<Type>) -> Self {
        Self { next_var, type_env }
    }

    fn fresh_var(&mut self, name: &str) -> Variable {
        let var = Variable {
            name: name.to_owned(),
            id: Some(self.next_var),
        };
        self.next_var += 1;
        var
    }

    pub fn convert(&mut self, expr: Expr, anfs: &mut ANFs) {
        let t = Type::get_type(&self.type_env, &expr).unwrap();
        match expr {
            Expr::Var(var) => {
                anfs.value = Some(Value::Var(var));
            }
            Expr::Abs(var, expr) => {
                let f = self.fresh_var("f");
                self.type_env.push(t);
                let mut anf = ANFs {
                    anfs: Vec::new(),
                    value: None,
                };
                self.convert(*expr, &mut anf);
                anfs.anfs.push(ANF::Fun(f.clone(), vec![var], anf));
                anfs.value = Some(Value::Var(f));
            }
            Expr::App(expr1, expr2) => {
                self.convert(*expr1, anfs);
                let f = anfs.value.clone();
                self.convert(*expr2, anfs);
                let x = anfs.value.clone();
                match f {
                    Some(Value::Var(f)) => {
                        let y = self.fresh_var("y");
                        self.type_env.push(t);
                        anfs.anfs.push(ANF::App(y.clone(), f, vec![x.unwrap()]));
                        anfs.value = Some(Value::Var(y));
                    }
                    _ => panic!("Must be named value!"),
                }
            }
            Expr::Number(n) => {
                anfs.value = Some(Value::Number(n));
            }
            Expr::BOp(op, expr1, expr2) => {
                self.convert(*expr1, anfs);
                let x = anfs.value.clone();
                self.convert(*expr2, anfs);
                let y = anfs.value.clone();
                let z = self.fresh_var("z");
                self.type_env.push(t);
                anfs.anfs
                    .push(ANF::BOp(z.clone(), op, x.unwrap(), y.unwrap()));
                anfs.value = Some(Value::Var(z));
            }
        }
    }

    pub fn closure_conversion(&mut self, anfs: ANFs) -> ANFs {
        let mut new_anfs = ANFs {
            anfs: Vec::new(),
            value: None,
        };
        for anf in anfs.anfs {
            match anf {
                ANF::Fun(var, args, funbody_anfs) => {
                    let env_var = self.fresh_var("env");
                    let fun_type = self.type_env[var.id.unwrap() as usize].simplify();
                    let mut env_var_type = vec![fun_type.clone()];
                    let new_funname = self.fresh_var(&var.name);
                    let free_vars =
                        funbody_anfs.free_vars(&mut args.iter().map(|x| x.id.unwrap()).collect());
                    for free_var in &free_vars {
                        env_var_type.push(self.type_env[free_var.id.unwrap() as usize].clone());
                    }
                    self.type_env.push(Type::Tuple(env_var_type.clone()));
                    self.type_env.push(Type::Arrow(
                        Box::new(Type::Tuple(env_var_type.clone())),
                        Box::new(fun_type),
                    ));
                    self.type_env[var.id.unwrap() as usize] = Type::Tuple(env_var_type);
                    let mut funbody_anfs = self.closure_conversion(funbody_anfs);
                    for i in 0..free_vars.len() {
                        funbody_anfs.anfs.insert(
                            0,
                            ANF::Project(free_vars[i].clone(), env_var.clone(), i + 1),
                        );
                    }
                    let mut new_args = args;
                    new_args.insert(0, env_var);
                    new_anfs
                        .anfs
                        .push(ANF::Fun(new_funname.clone(), new_args, funbody_anfs));
                    let mut free_vars: Vec<Value> =
                        free_vars.into_iter().map(|x| Value::Var(x)).collect();
                    free_vars.insert(0, Value::Global(new_funname));
                    new_anfs.anfs.push(ANF::Tuple(var, free_vars))
                }
                ANF::App(var, func_var, args) => {
                    let ptr = self.fresh_var(&func_var.name);
                    match dbg!(self.type_env.get(func_var.id.unwrap() as usize)) {
                        Some(Type::Tuple(t)) => self.type_env.push(t[0].clone()),
                        _ => unreachable!(),
                    }
                    new_anfs
                        .anfs
                        .push(ANF::Project(ptr.clone(), func_var.clone(), 0));
                    let mut new_args = args;
                    new_args.insert(0, Value::Var(func_var));
                    new_anfs.anfs.push(ANF::App(var, ptr, new_args))
                }
                _ => new_anfs.anfs.push(anf),
            }
        }
        new_anfs.value = anfs.value;
        new_anfs
    }
}
