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
}

impl Expr {
    pub fn is_value(&self) -> bool {
        match self {
            Expr::Int(_) | Expr::Bool(_) | Expr::Fn { arg: _, body: _ } => true,
            _ => false,
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
