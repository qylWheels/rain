use crate::ast::{BinOp, Expr};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
}

#[derive(Clone, Debug)]
pub enum TypeError {
    UnboundVariable(String),
    IfGuardError,
    IfBranchError,
    BinOpError,
}

#[derive(Debug)]
struct TypeEnvironment {
    type_env: HashMap<String, Type>,
}

impl TypeEnvironment {
    fn new() -> Self {
        Self {
            type_env: HashMap::new(),
        }
    }

    fn lookup(&self, varname: &str) -> Option<Type> {
        self.type_env.get(varname).copied()
    }

    fn extend(&mut self, varname: &str, typ: Type) {
        self.type_env.insert(varname.into(), typ);
    }
}

#[derive(Debug)]
pub struct TypeChecker {
    type_env: TypeEnvironment,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            type_env: TypeEnvironment::new(),
        }
    }

    pub fn typecheck(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        let expr = expr.clone();
        match expr {
            Expr::Int(_) => Ok(Type::Int),
            Expr::Bool(_) => Ok(Type::Bool),
            Expr::Var(varname) => Ok(self
                .type_env
                .lookup(&varname)
                .ok_or(TypeError::UnboundVariable(varname))?),
            Expr::Let { x, e1, e2 } => {
                let e1_type = self.typecheck(&e1)?;
                self.type_env.extend(&x, e1_type);
                self.typecheck(&e2)
            }
            Expr::If { guard, e1, e2 } => {
                let guard_type = self.typecheck(&guard)?;
                if guard_type != Type::Bool {
                    Err(TypeError::IfGuardError)
                } else {
                    let e1_type = self.typecheck(&e1)?;
                    let e2_type = self.typecheck(&e2)?;
                    if e1_type != e2_type {
                        Err(TypeError::IfBranchError)
                    } else {
                        Ok(e1_type)
                    }
                }
            }
            Expr::Bin { op, e1, e2 } => {
                let e1_type = self.typecheck(&e1)?;
                let e2_type = self.typecheck(&e2)?;
                match (op, e1_type, e2_type) {
                    (BinOp::Plus, Type::Int, Type::Int) => Ok(Type::Int),
                    (BinOp::Times, Type::Int, Type::Int) => Ok(Type::Int),
                    (BinOp::Le, Type::Int, Type::Int) => Ok(Type::Bool),
                    _ => Err(TypeError::BinOpError),
                }
            }
        }
    }
}
