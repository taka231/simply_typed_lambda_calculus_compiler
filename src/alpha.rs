use std::{cell::RefCell, rc::Rc};

use crate::ast::{Expr, Variable};

#[derive(Debug, Clone, PartialEq)]
enum AlphaConvMap {
    Nil,
    Cons(String, u32, Box<AlphaConvMap>),
}

impl AlphaConvMap {
    fn search(&self, value: &str) -> Option<u32> {
        match self {
            AlphaConvMap::Nil => None,
            AlphaConvMap::Cons(name, id, env) => {
                if name == value {
                    Some(*id)
                } else {
                    env.search(value)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlphaConvEnv {
    map: AlphaConvMap,
    id: Rc<RefCell<u32>>,
}

impl AlphaConvEnv {
    pub fn new() -> AlphaConvEnv {
        AlphaConvEnv {
            map: AlphaConvMap::Nil,
            id: Rc::new(RefCell::new(0)),
        }
    }

    pub fn id(&self) -> u32 {
        *self.id.borrow()
    }

    fn get_new_id(&self) -> u32 {
        let new_id = *self.id.borrow();
        *self.id.borrow_mut() += 1;
        new_id
    }

    fn add_variable(&self, var_name: String) -> AlphaConvEnv {
        let new_id = self.get_new_id();
        AlphaConvEnv {
            map: AlphaConvMap::Cons(var_name, new_id, Box::new(self.map.clone())),
            id: Rc::clone(&self.id),
        }
    }

    pub fn alpha_conversion(&self, expr: Expr) -> Option<Expr> {
        match expr {
            Expr::Var(var) => {
                let id = self.map.search(&var.name)?;
                Some(Expr::Var(Variable {
                    name: var.name,
                    id: Some(id),
                }))
            }
            Expr::Abs(var, expr) => {
                let new_alpha_conv_env = self.add_variable(var.name.clone());
                let id = new_alpha_conv_env.map.search(&var.name);
                let expr = new_alpha_conv_env.alpha_conversion(*expr)?;
                Some(Expr::Abs(Variable { name: var.name, id }, Box::new(expr)))
            }
            Expr::App(expr1, expr2) => {
                let expr1 = self.alpha_conversion(*expr1)?;
                let expr2 = self.alpha_conversion(*expr2)?;
                Some(Expr::App(Box::new(expr1), Box::new(expr2)))
            }
            Expr::BOp(op, expr1, expr2) => {
                let expr1 = self.alpha_conversion(*expr1)?;
                let expr2 = self.alpha_conversion(*expr2)?;
                Some(Expr::BOp(op, Box::new(expr1), Box::new(expr2)))
            }
            Expr::Number(n) => Some(Expr::Number(n)),
        }
    }
}
