use crate::ast::{BinOp, Expr};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub enum InterpretError {
    UnboundValue(String),
    NotAValue,
}

#[derive(Clone, Debug)]
struct VariableNameGenerator {
    counter: u32,
}

impl VariableNameGenerator {
    fn new() -> Self {
        VariableNameGenerator { counter: 0 }
    }

    fn next(&mut self) -> String {
        self.counter += 1;
        format!("${}", self.counter)
    }
}

#[derive(Clone, Debug)]
pub struct Interpreter {
    var_name_generator: VariableNameGenerator,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            var_name_generator: VariableNameGenerator::new(),
        }
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Expr, InterpretError> {
        let result = match expr {
            Expr::Int(_) | Expr::Bool(_) => expr.clone(),
            Expr::Var(x) => Err(InterpretError::UnboundValue(x.clone()))?,
            Expr::Let { x, e1, e2 } => self.eval_let(x, e1, e2)?,
            Expr::If { guard, e1, e2 } => self.eval_if(guard, e1, e2)?,
            Expr::Bin { op, e1, e2 } => self.eval_bin(op.clone(), e1, e2)?,
            Expr::Fn { arg: _, body: _ } => expr.clone(),
            Expr::Apply { func, arg } => self.eval_apply(func, arg)?,
        };
        if !result.is_value() {
            self.eval(&result)
        } else {
            Ok(result)
        }
    }

    fn freevars(&self, expr: &Expr) -> HashSet<String> {
        match expr {
            Expr::Int(_) | Expr::Bool(_) => HashSet::new(),
            Expr::Var(x) => [x.clone()].into(),
            Expr::Let { x, e1, e2 } => {
                &self.freevars(e1) | &(&(self.freevars(e2)) ^ &[x.clone()].into())
            }
            Expr::If { guard, e1, e2 } => {
                &(&self.freevars(guard) | &self.freevars(e1)) | &self.freevars(e2)
            }
            Expr::Bin { op: _, e1, e2 } => &self.freevars(e1) | &self.freevars(e2),
            Expr::Fn { arg, body } => &self.freevars(body) ^ &[arg.clone()].into(),
            Expr::Apply { func, arg } => &self.freevars(func) | &self.freevars(arg),
        }
    }

    fn replace(
        &self,
        expr: &Expr,
        old_varname: &str,
        new_varname: &str,
    ) -> Result<Expr, InterpretError> {
        match expr {
            Expr::Int(_) | Expr::Bool(_) => Ok(expr.clone()),
            Expr::Var(varname) => {
                if varname == old_varname {
                    Ok(Expr::Var(new_varname.to_owned()))
                } else {
                    Ok(expr.clone())
                }
            }
            Expr::Let { x, e1, e2 } => {
                if x == old_varname {
                    Ok(Expr::Let {
                        x: new_varname.to_owned(),
                        e1: e1.clone(),
                        e2: Box::new(self.replace(e2, old_varname, new_varname)?),
                    })
                } else {
                    Ok(Expr::Let {
                        x: x.clone(),
                        e1: e1.clone(),
                        e2: Box::new(self.replace(e2, old_varname, new_varname)?),
                    })
                }
            }
            Expr::If { guard, e1, e2 } => Ok(Expr::If {
                guard: Box::new(self.replace(guard, old_varname, new_varname)?),
                e1: Box::new(self.replace(e1, old_varname, new_varname)?),
                e2: Box::new(self.replace(e2, old_varname, new_varname)?),
            }),
            Expr::Bin { op, e1, e2 } => Ok(Expr::Bin {
                op: op.clone(),
                e1: Box::new(self.replace(e1, old_varname, new_varname)?),
                e2: Box::new(self.replace(e2, old_varname, new_varname)?),
            }),
            Expr::Fn { arg, body } => {
                if arg == old_varname {
                    Ok(Expr::Fn {
                        arg: new_varname.to_owned(),
                        body: Box::new(self.replace(body, old_varname, new_varname)?),
                    })
                } else {
                    Ok(Expr::Fn {
                        arg: arg.clone(),
                        body: Box::new(self.replace(body, old_varname, new_varname)?),
                    })
                }
            }
            Expr::Apply { func, arg } => Ok(Expr::Apply {
                func: Box::new(self.replace(func, old_varname, new_varname)?),
                arg: Box::new(self.replace(arg, old_varname, new_varname)?),
            }),
        }
    }

    fn substitute(
        &mut self,
        expr: &Expr,
        value: &Expr,
        varname: &str,
    ) -> Result<Expr, InterpretError> {
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
                        let freevars = self.freevars(&value);
                        if !freevars.contains(&arg) {
                            Ok(Expr::Fn {
                                arg,
                                body: Box::new(self.substitute(&body, &value, varname)?),
                            })
                        } else {
                            let fresh = self.var_name_generator.next();
                            let replaced = self.replace(
                                &Expr::Fn {
                                    arg: arg.clone(),
                                    body: body.clone(),
                                },
                                &arg,
                                &fresh,
                            )?;
                            self.substitute(&replaced, &value, varname)
                        }
                    }
                }
                Expr::Apply { func, arg } => Ok(Expr::Apply {
                    func: Box::new(self.substitute(&func, &value, varname)?),
                    arg: Box::new(self.substitute(&arg, &value, varname)?),
                }),
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

    fn eval_apply(&mut self, func: &Expr, arg: &Expr) -> Result<Expr, InterpretError> {
        let func_final = self.eval(func)?;
        let val = self.eval(arg)?;
        match func_final {
            Expr::Fn { arg, body } => self.substitute(&body, &val, &arg),
            _ => unreachable!(),
        }
    }
}
