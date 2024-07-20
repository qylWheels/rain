use crate::ast::{BinOp, Expr};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub enum InterpretError {
    UnboundValue(String),
    NotAValue,
}

#[derive(Clone, Debug)]
pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Interpreter
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Expr, InterpretError> {
        let expr = expr.clone();
        match expr {
            Expr::Int(_) | Expr::Bool(_) => Ok(expr),
            Expr::Var(x) => Err(InterpretError::UnboundValue(x)),
            Expr::Let { x, e1, e2 } => self.eval_let(&x, &e1, &e2),
            Expr::If { guard, e1, e2 } => self.eval_if(&guard, &e1, &e2),
            Expr::Bin { op, e1, e2 } => self.eval_bin(op, &e1, &e2),
            Expr::Fn { arg: _, body: _ } => Ok(expr),
        }
    }

    fn freevars(&self, expr: &Expr) -> HashSet<String> {
        let expr = expr.clone();
        match expr {
            Expr::Int(_) | Expr::Bool(_) => HashSet::new(),
            Expr::Var(x) => [x].into(),
            Expr::Let { x, e1, e2 } => &self.freevars(&e1) | &(&(self.freevars(&e2)) ^ &[x].into()),
            Expr::If { guard, e1, e2 } => {
                &(&self.freevars(&guard) | &self.freevars(&e1)) | &self.freevars(&e2)
            }
            Expr::Bin { op: _, e1, e2 } => &self.freevars(&e1) | &self.freevars(&e2),
            Expr::Fn { arg, body } => &self.freevars(&body) ^ &[arg].into(),
        }
    }

    fn substitute(&self, expr: &Expr, value: &Expr, varname: &str) -> Result<Expr, InterpretError> {
        let expr = expr.clone();
        let value = value.clone();
        match value.is_value() {
            true => match expr {
                Expr::Int(_) | Expr::Bool(_) => Ok(expr),
                Expr::Var(x) => {
                    if x == varname {
                        Ok(value)
                    } else {
                        Ok(Expr::Var(x))
                    }
                }
                Expr::Let { x, e1, e2 } => {
                    let new_e1 = self.substitute(&e1, &value, varname)?;
                    if x == varname {
                        Ok(Expr::Let {
                            x,
                            e1: Box::new(new_e1),
                            e2,
                        })
                    } else {
                        Ok(Expr::Let {
                            x,
                            e1: Box::new(new_e1),
                            e2: Box::new(self.substitute(&e2, &value, varname)?),
                        })
                    }
                }
                Expr::If { guard, e1, e2 } => Ok(Expr::If {
                    guard: Box::new(self.substitute(&guard, &value, varname)?),
                    e1: Box::new(self.substitute(&e1, &value, varname)?),
                    e2: Box::new(self.substitute(&e2, &value, varname)?),
                }),
                Expr::Bin { op, e1, e2 } => Ok(Expr::Bin {
                    op: op,
                    e1: Box::new(self.substitute(&e1, &value, varname)?),
                    e2: Box::new(self.substitute(&e2, &value, varname)?),
                }),
                Expr::Fn { arg, body } => {
                    if arg == varname {
                        Ok(Expr::Fn { arg, body })
                    } else {
                        // for now, "value" is always a value, so FV(value) is always {}.
                        let freevars = self.freevars(&value);
                        if !freevars.contains(&arg) {
                            Ok(Expr::Fn {
                                arg,
                                body: Box::new(self.substitute(&body, &value, varname)?),
                            })
                        } else {
                            unimplemented!()
                        }
                    }
                }
            },
            false => Err(InterpretError::NotAValue),
        }
    }

    fn eval_let(&mut self, x: &str, e1: &Expr, e2: &Expr) -> Result<Expr, InterpretError> {
        let v1 = self.eval(e1)?;
        let expr = self.substitute(e2, &v1, x)?;
        self.eval(&expr)
    }

    fn eval_if(&mut self, guard: &Expr, e1: &Expr, e2: &Expr) -> Result<Expr, InterpretError> {
        let guard_value = self.eval(guard)?;
        match guard_value {
            Expr::Bool(b) => {
                if b == true {
                    Ok(self.eval(e1)?)
                } else {
                    Ok(self.eval(e2)?)
                }
            }
            _ => unreachable!(),
        }
    }

    fn eval_bin(&mut self, op: BinOp, e1: &Expr, e2: &Expr) -> Result<Expr, InterpretError> {
        let v1 = self.eval(e1)?;
        let v2 = self.eval(e2)?;
        match (v1, v2) {
            (Expr::Int(v1), Expr::Int(v2)) => match op {
                BinOp::Plus => Ok(Expr::Int(v1 + v2)),
                BinOp::Times => Ok(Expr::Int(v1 * v2)),
                BinOp::Le => Ok(Expr::Bool(v1 <= v2)),
            },
            _ => unreachable!(),
        }
    }
}
