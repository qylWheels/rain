use crate::ast::{BinOp, Expr};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Display,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MonoType {
    Int,
    Bool,
    Fn(Box<MonoType>, Box<MonoType>), // t1 -> t2
    TypeVariable(String),             // '1, '2, '3...
}

impl Display for MonoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn helper(f: &mut std::fmt::Formatter<'_>, monotype: &MonoType) -> std::fmt::Result {
            let monotype = monotype.clone();
            match monotype {
                MonoType::Int => write!(f, "Int"),
                MonoType::Bool => write!(f, "Bool"),
                MonoType::TypeVariable(x) => write!(f, "{x}"),
                MonoType::Fn(i, o) => {
                    helper(f, &i)?;
                    write!(f, " -> ")?;
                    helper(f, &o)
                }
            }
        }

        helper(f, self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PolyType {
    bounded_type_vars: Vec<String>,
    typ: MonoType,
}

impl Display for PolyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn helper(f: &mut std::fmt::Formatter<'_>, polytype: &PolyType) -> std::fmt::Result {
            let polytype = polytype.clone();
            for bounded in polytype.bounded_type_vars {
                write!(f, "{bounded} ")?;
            }
            write!(f, ". ")?;
            println!("{}", polytype.typ);
            Ok(())
        }

        helper(f, self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    MonoType(MonoType),
    PolyType(PolyType),
}

#[derive(Clone, Debug)]
pub enum TypeError {
    UnboundVariable(String),
    IfGuardError(MonoType),
    IfBranchError(MonoType, MonoType),
    BinOpError,
    UnsolvableConstraints(MonoType, MonoType),
}

#[derive(Debug)]
struct TypeVariableNameGenerator {
    counter: u32,
}

impl TypeVariableNameGenerator {
    fn new() -> Self {
        Self { counter: 0 }
    }

    fn next(&mut self) -> String {
        self.counter += 1;
        format!("'{}", self.counter)
    }
}

type TypeEnvironment = HashMap<String, Type>;
type TypeConstraints = VecDeque<(MonoType, MonoType)>;
type Substitutions = VecDeque<(String, MonoType)>;

#[derive(Debug)]
pub struct TypeChecker {
    /// Type variable name generator.
    type_var_name_generator: TypeVariableNameGenerator,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            type_var_name_generator: TypeVariableNameGenerator::new(),
        }
    }

    pub fn type_check(&mut self, expr: &Expr) -> Result<MonoType, TypeError> {
        let (t, c) = self.build_constraints(&HashMap::new(), expr)?;
        // println!("t = {t}, c = {c:?}");
        let substitutions = self.unify(&c)?;
        // println!("substitutions = {substitutions:?}");
        Self::apply_substitutions_for_monotype(&t, &substitutions)
    }

    fn build_constraints(
        &mut self,
        env: &TypeEnvironment,
        expr: &Expr,
    ) -> Result<(MonoType, TypeConstraints), TypeError> {
        match expr {
            Expr::Int(_) => Ok((MonoType::Int, TypeConstraints::new())),
            Expr::Bool(_) => Ok((MonoType::Bool, TypeConstraints::new())),
            Expr::Var(x) => {
                let typ = env
                    .get(x)
                    .cloned()
                    .ok_or(TypeError::UnboundVariable(x.clone()))?;
                let monotype = match typ {
                    Type::PolyType(polytype) => self.instantiate(&polytype)?,
                    Type::MonoType(monotype) => monotype,
                };
                Ok((monotype, TypeConstraints::new()))
            }
            Expr::Bin { op, e1, e2 } => {
                let fresh = self.type_var_name_generator.next();
                let (t1, c1) = self.build_constraints(env, e1)?;
                let (t2, c2) = self.build_constraints(env, e2)?;
                let mut constraints = TypeConstraints::new();
                constraints.extend(c1);
                constraints.extend(c2);
                // TODO: Let the initial environment has binding of the boolean operators.
                let result_type = match op {
                    BinOp::Plus | BinOp::Times => MonoType::Int,
                    BinOp::Le => MonoType::Bool,
                };
                constraints.extend([
                    (t1, MonoType::Int),
                    (t2, MonoType::Int),
                    (MonoType::TypeVariable(fresh.clone()), result_type),
                ]);
                Ok((MonoType::TypeVariable(fresh.clone()), constraints))
            }
            Expr::If { guard, e1, e2 } => {
                let fresh = self.type_var_name_generator.next();
                let (tg, cg) = self.build_constraints(env, guard)?;
                let (t1, c1) = self.build_constraints(env, e1)?;
                let (t2, c2) = self.build_constraints(env, e2)?;
                let mut constraints = VecDeque::new();
                constraints.extend(cg);
                constraints.extend(c1);
                constraints.extend(c2);
                constraints.extend([
                    (tg, MonoType::Bool),
                    (MonoType::TypeVariable(fresh.clone()), t1),
                    (MonoType::TypeVariable(fresh.clone()), t2),
                ]);
                Ok((MonoType::TypeVariable(fresh.clone()), constraints))
            }
            Expr::Fn { arg, body } => {
                let fresh = self.type_var_name_generator.next();
                let mut new_env = env.clone();
                new_env.insert(
                    arg.clone(),
                    Type::MonoType(MonoType::TypeVariable(fresh.clone())),
                );
                let (t, c) = self.build_constraints(&new_env, body)?;
                Ok((
                    MonoType::Fn(Box::new(MonoType::TypeVariable(fresh.clone())), Box::new(t)),
                    c,
                ))
            }
            Expr::Let { x, e1, e2 } => {
                let (t1, c1) = self.build_constraints(env, e1)?;
                let new_env = self.generalize(&c1, env, (x, &t1))?;
                let (t2, c2) = self.build_constraints(&new_env, e2)?;
                let mut constraints = TypeConstraints::new();
                constraints.extend(c1);
                constraints.extend(c2);
                Ok((t2, constraints))
            }
        }
    }

    fn is_appear_in(&self, type_var_name: &str, monotype: &MonoType) -> bool {
        let monotype = monotype.clone();
        match monotype {
            MonoType::Int | MonoType::Bool => false,
            MonoType::TypeVariable(x) => {
                if &x == type_var_name {
                    true
                } else {
                    false
                }
            }
            MonoType::Fn(i, o) => {
                self.is_appear_in(type_var_name, &i) || self.is_appear_in(type_var_name, &o)
            }
        }
    }

    /// Substitutions must have form like `{<some>/'a}`.
    fn unify(&mut self, constraints: &TypeConstraints) -> Result<Substitutions, TypeError> {
        if constraints.is_empty() {
            Ok(Substitutions::new())
        } else {
            let (lhs, rhs) = &constraints[0];
            match (lhs, rhs) {
                (MonoType::Int, MonoType::Int) | (MonoType::Bool, MonoType::Bool) => {
                    self.unify(&constraints.iter().skip(1).map(|c| c.clone()).collect())
                }
                (MonoType::TypeVariable(x), MonoType::TypeVariable(y)) if x == y => {
                    self.unify(&constraints.iter().skip(1).map(|c| c.clone()).collect())
                }
                (MonoType::TypeVariable(x), monotype) if !self.is_appear_in(&x, &monotype) => {
                    let mut substitutions =
                        Substitutions::from_iter([(x.clone(), monotype.clone())]);
                    substitutions.extend(
                        self.unify(
                            &constraints
                                .iter()
                                .skip(1)
                                .map(|(lhs, rhs)| {
                                    (
                                        // FIXME: Do not `unwrap()`.
                                        Self::apply_substitutions_for_monotype(lhs, &substitutions)
                                            .unwrap(),
                                        Self::apply_substitutions_for_monotype(rhs, &substitutions)
                                            .unwrap(),
                                    )
                                })
                                .collect(),
                        )?,
                    );
                    Ok(substitutions)
                }
                (monotype, MonoType::TypeVariable(x)) if !self.is_appear_in(&x, &monotype) => {
                    let mut substitutions =
                        Substitutions::from_iter([(x.clone(), monotype.clone())]);
                    substitutions.extend(
                        self.unify(
                            &constraints
                                .iter()
                                .skip(1)
                                .map(|(lhs, rhs)| {
                                    (
                                        // FIXME: Do not `unwrap()`.
                                        Self::apply_substitutions_for_monotype(lhs, &substitutions)
                                            .unwrap(),
                                        Self::apply_substitutions_for_monotype(rhs, &substitutions)
                                            .unwrap(),
                                    )
                                })
                                .collect(),
                        )?,
                    );
                    Ok(substitutions)
                }
                (MonoType::Fn(i1, o1), MonoType::Fn(i2, o2)) => {
                    let mut new_constraints: TypeConstraints =
                        constraints.iter().skip(1).map(|c| c.clone()).collect();
                    new_constraints.push_front((*o1.clone(), *o2.clone()));
                    new_constraints.push_front((*i1.clone(), *i2.clone()));
                    self.unify(&new_constraints)
                }
                _ => {
                    println!("{constraints:?}");
                    Err(TypeError::UnsolvableConstraints(lhs.clone(), rhs.clone()))
                }
            }
        }
    }

    fn apply_substitutions_for_monotype(
        monotype: &MonoType,
        substitutions: &Substitutions,
    ) -> Result<MonoType, TypeError> {
        match monotype {
            MonoType::Int | MonoType::Bool => Ok(monotype.clone()),
            MonoType::TypeVariable(x) => {
                for (type_var, monotype) in substitutions.iter() {
                    if x == type_var {
                        return Self::apply_substitutions_for_monotype(monotype, substitutions);
                    }
                }
                Ok(monotype.clone())
            }
            MonoType::Fn(i, o) => Ok(MonoType::Fn(
                Box::new(Self::apply_substitutions_for_monotype(i, substitutions)?),
                Box::new(Self::apply_substitutions_for_monotype(o, substitutions)?),
            )),
        }
    }

    fn apply_substitutions_for_polytype(
        polytype: &PolyType,
        substitutions: &Substitutions,
    ) -> Result<PolyType, TypeError> {
        let bounded_type_vars: HashSet<String> =
            HashSet::from_iter(polytype.bounded_type_vars.clone());
        let filtered_substitutions = substitutions
            .iter()
            .filter(|(type_var, _)| !bounded_type_vars.contains(type_var))
            .map(|(type_var, monotype)| (type_var.clone(), monotype.clone()))
            .collect::<Substitutions>();
        Ok(PolyType {
            bounded_type_vars: polytype.bounded_type_vars.clone(),
            typ: Self::apply_substitutions_for_monotype(&polytype.typ, &filtered_substitutions)?,
        })
    }

    fn instantiate(&mut self, polytype: &PolyType) -> Result<MonoType, TypeError> {
        let bounded_type_var_count = polytype.bounded_type_vars.len();
        let substitutions = (0..bounded_type_var_count)
            .map(|i| {
                (
                    polytype.bounded_type_vars[i].clone(),
                    MonoType::TypeVariable(self.type_var_name_generator.next()),
                )
            })
            .collect();
        Self::apply_substitutions_for_monotype(&polytype.typ, &substitutions)
    }

    fn generalize(
        &mut self,
        constraints: &TypeConstraints,
        env: &TypeEnvironment,
        var_and_type: (&String, &MonoType),
    ) -> Result<TypeEnvironment, TypeError> {
        let substitutions = self.unify(constraints)?;
        let u = Self::apply_substitutions_for_monotype(var_and_type.1, &substitutions)?;
        let mut new_env: TypeEnvironment = env
            .iter()
            .map(|(varname, typ)| {
                // FIXME: Do not `unwrap()`.
                let typ = match typ {
                    Type::MonoType(monotype) => Type::MonoType(
                        Self::apply_substitutions_for_monotype(&monotype, &substitutions).unwrap(),
                    ),
                    Type::PolyType(polytype) => Type::PolyType(
                        Self::apply_substitutions_for_polytype(&polytype, &substitutions).unwrap(),
                    ),
                };
                (varname.clone(), typ)
            })
            .collect();
        let free_vars_in_u = self.free_type_vars(&Type::MonoType(u.clone()));
        let free_vars_in_new_env: HashSet<String> = new_env
            .values()
            .map(|typ| self.free_type_vars(typ))
            .flatten()
            .collect();
        let diff: HashSet<String> = free_vars_in_u
            .difference(&free_vars_in_new_env)
            .map(|s| s.clone())
            .collect();
        if diff.is_empty() {
            new_env.insert(var_and_type.0.clone(), Type::MonoType(u.clone()));
            Ok(new_env)
        } else {
            new_env.insert(
                var_and_type.0.clone(),
                Type::PolyType(PolyType {
                    bounded_type_vars: Vec::from_iter(diff),
                    typ: u.clone(),
                }),
            );
            Ok(new_env)
        }
    }

    fn free_type_vars(&self, typ: &Type) -> HashSet<String> {
        let typ = typ.clone();
        match typ {
            Type::MonoType(monotype) => match monotype {
                MonoType::Int | MonoType::Bool => HashSet::new(),
                MonoType::TypeVariable(x) => HashSet::from([x]),
                MonoType::Fn(i, o) => {
                    &self.free_type_vars(&Type::MonoType(*i))
                        | &self.free_type_vars(&Type::MonoType(*o))
                }
            },
            Type::PolyType(polytype) => {
                let free = self.free_type_vars(&Type::MonoType(polytype.typ));
                let bounded = HashSet::from_iter(polytype.bounded_type_vars);
                free.difference(&bounded).map(|s| s.clone()).collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_unify() {
        use super::*;

        // 'a = 'd -> 'e
        // 'c = int -> 'd
        // int -> int -> int = 'b -> 'c

        let mut type_checker = TypeChecker::new();
        let constraints = VecDeque::from_iter([
            (
                MonoType::TypeVariable("'a".to_owned()),
                MonoType::Fn(
                    Box::new(MonoType::TypeVariable("'d".to_owned())),
                    Box::new(MonoType::TypeVariable("'e".to_owned())),
                ),
            ),
            (
                MonoType::TypeVariable("'c".to_owned()),
                MonoType::Fn(
                    Box::new(MonoType::Int),
                    Box::new(MonoType::TypeVariable("'d".to_owned())),
                ),
            ),
            (
                MonoType::Fn(
                    Box::new(MonoType::Int),
                    Box::new(MonoType::Fn(
                        Box::new(MonoType::Int),
                        Box::new(MonoType::Int),
                    )),
                ),
                MonoType::Fn(
                    Box::new(MonoType::TypeVariable("'b".to_owned())),
                    Box::new(MonoType::TypeVariable("'c".to_owned())),
                ),
            ),
        ]);
        let substitutions = type_checker.unify(&constraints).unwrap();

        println!("{substitutions:#?}");
    }
}
