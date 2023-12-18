use std::{cell::RefCell, rc::Rc};

use crate::ast::Expr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Arrow(Box<Type>, Box<Type>),
    TVar(usize, Rc<RefCell<Option<Type>>>),
}

impl Type {
    pub fn simplify(&self) -> Self {
        match self {
            Type::TVar(n, r) => match &*r.borrow() {
                Some(t) => t.simplify(),
                None => Type::TVar(*n, Rc::clone(r)),
            },
            Type::Arrow(t1, t2) => Type::Arrow(Box::new(t1.simplify()), Box::new(t2.simplify())),
            _ => self.clone(),
        }
    }

    pub fn get_type(env: &[Type], expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Var(var) => env.get(var.id).map(|t| t.simplify()),
            Expr::Abs(var, expr) => {
                let t = env.get(var.id)?.simplify();
                let t2 = Self::get_type(env, expr)?;
                Some(Type::Arrow(Box::new(t), Box::new(t2)))
            }
            Expr::App(expr1, _) => {
                let t1 = Self::get_type(env, expr1)?;
                match t1 {
                    Type::Arrow(_, t12) => Some(t12.simplify()),
                    _ => None,
                }
            }
            Expr::Number(_) => Some(Type::Int),
            Expr::BOp(_, _, _) => Some(Type::Int),
        }
    }
}

pub struct TypeInfer {
    pub next_tvar: usize,
    pub env: Vec<Type>,
}

impl TypeInfer {
    pub fn new(next_tvar: usize) -> Self {
        Self {
            next_tvar,
            env: (0..next_tvar)
                .map(|n| Type::TVar(n, Rc::new(RefCell::new(None))))
                .collect(),
        }
    }

    fn unify(&mut self, t1: &Type, t2: &Type) -> bool {
        let t1 = t1.simplify();
        let t2 = t2.simplify();
        match (t1, t2) {
            (Type::Int, Type::Int) => true,
            (Type::Arrow(t11, t12), Type::Arrow(t21, t22)) => {
                self.unify(&t11, &t21) && self.unify(&t12, &t22)
            }
            (Type::TVar(n1, _), Type::TVar(n2, _)) if n1 == n2 => true,
            (Type::TVar(_, r), t) | (t, Type::TVar(_, r)) => {
                *r.borrow_mut() = Some(t.clone());
                true
            }
            _ => false,
        }
    }

    fn new_tvar(&mut self) -> Type {
        let t = Type::TVar(self.next_tvar, Rc::new(RefCell::new(None)));
        self.next_tvar += 1;
        self.env.push(t.clone());
        t
    }

    pub fn type_infer(&mut self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Var(var) => self.env.get(var.id).map(|t| t.clone()),
            Expr::Abs(var, expr) => {
                let t = self.env.get(var.id)?.clone();
                let t2 = self.type_infer(expr)?;
                Some(Type::Arrow(Box::new(t), Box::new(t2)))
            }
            Expr::App(e1, e2) => {
                let t1 = self.type_infer(e1)?;
                let t2 = self.type_infer(e2)?;
                let ret_type = self.new_tvar();
                if self.unify(&t1, &Type::Arrow(Box::new(t2), Box::new(ret_type.clone()))) {
                    Some(ret_type)
                } else {
                    None
                }
            }
            Expr::Number(_) => Some(Type::Int),
            Expr::BOp(_, e1, e2) => {
                let t1 = self.type_infer(e1)?;
                let t2 = self.type_infer(e2)?;
                if self.unify(&t1, &Type::Int) && self.unify(&t2, &Type::Int) {
                    Some(Type::Int)
                } else {
                    None
                }
            }
        }
    }
}
