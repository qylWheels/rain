use std::fmt::{write, Display};

#[derive(Clone, Copy, Debug)]
pub enum BinOp {
    Plus,
    Times,
    Le,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Var(String),
    Let {
        x: String,
        e1: Box<Expr>,
        e2: Box<Expr>,
    },
    If {
        guard: Box<Expr>,
        e1: Box<Expr>,
        e2: Box<Expr>,
    },
    Bin {
        op: BinOp,
        e1: Box<Expr>,
        e2: Box<Expr>,
    },
    Fn {
        arg: String,
        body: Box<Expr>,
    },
    Apply {
        func: Box<Expr>,
        arg: Box<Expr>,
    },
}

impl Expr {
    pub fn is_value(&self) -> bool {
        match self {
            Expr::Int(_) | Expr::Bool(_) | Expr::Fn { arg: _, body: _ } => true,
            _ => false,
        }
    }
}

// Only for printing `value`.
impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Int(i) => write!(f, "{i}"),
            Expr::Bool(b) => write!(f, "{b}"),
            Expr::Fn { arg: _, body: _ } => write!(f, "<function>"),
            other => unreachable!("Expression must be a value, but it is {other:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Expr;

    #[test]
    fn test_is_value() {
        let e = Expr::Int(4);
        assert_eq!(e.is_value(), true);

        let e = Expr::Bool(true);
        assert_eq!(e.is_value(), true);

        let e = Expr::Var("x".into());
        assert_eq!(e.is_value(), false);
    }
}
