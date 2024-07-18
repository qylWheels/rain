use crate::ast::{BinOp, Expr};
use lazy_static::lazy_static;
use pest::{iterators::Pairs, pratt_parser::PrattParser};
use pest_derive::Parser;

lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc, Op};
        use Rule::*;

        PrattParser::new()
            .op(Op::infix(le, Assoc::Left))
            .op(Op::infix(plus, Assoc::Left))
            .op(Op::infix(times, Assoc::Left))
    };
}

#[derive(Parser)]
#[grammar = "pest/syntax.pest"]
pub struct RainParser;

impl RainParser {
    pub fn parse_expression(pairs: Pairs<Rule>) -> Expr {
        PRATT_PARSER
            .map_primary(|primary| match primary.as_rule() {
                Rule::int => Expr::Int(primary.as_str().parse().unwrap()),
                Rule::r#true => Expr::Bool(true),
                Rule::r#false => Expr::Bool(false),
                Rule::id => Expr::Var(primary.as_str().into()),
                Rule::let_expr => {
                    let mut inner = primary.into_inner();
                    let x = inner.next().unwrap().as_str();
                    let e1 = Self::parse_expression(inner.next().unwrap().into_inner());
                    let e2 = Self::parse_expression(inner.next().unwrap().into_inner());
                    Expr::Let {
                        x: x.into(),
                        e1: Box::new(e1),
                        e2: Box::new(e2),
                    }
                }
                Rule::r#if_expr => {
                    let mut inner = primary.into_inner();
                    let guard = Self::parse_expression(inner.next().unwrap().into_inner());
                    let e1 = Self::parse_expression(inner.next().unwrap().into_inner());
                    let e2 = Self::parse_expression(inner.next().unwrap().into_inner());
                    Expr::If {
                        guard: Box::new(guard),
                        e1: Box::new(e1),
                        e2: Box::new(e2),
                    }
                }
                Rule::binop_expr => Self::parse_expression(primary.into_inner()),
                Rule::expr => Self::parse_expression(primary.into_inner()),
                rule => unreachable!("rule = {rule:?}"),
            })
            .map_infix(|lhs, op, rhs| {
                let op = match op.as_rule() {
                    Rule::plus => BinOp::Plus,
                    Rule::times => BinOp::Times,
                    Rule::le => BinOp::Le,
                    _ => unreachable!(),
                };
                Expr::Bin {
                    op,
                    e1: Box::new(lhs),
                    e2: Box::new(rhs),
                }
            })
            .parse(pairs)
    }

    // /// Parse expression denoted by Rule::expr,
    // /// including "let expression" and "if expression" and so on.
    // pub fn parse_expression(pair: Pair<Rule>) -> Expr {
    //     let mut inner_expr = pair.into_inner();
    //     match inner_expr.next().unwrap().as_rule() {
    //         Rule::let_expr => {
    //             let inner_expr =
    //         },
    //         Rule::if_expr => todo!(),
    //         Rule::binop_expr => todo!(),
    //         Rule::int | Rule::r#true | Rule::r#false | Rule::id => {
    //             Self::parse_basic_expression(pair.into_inner())
    //         }
    //         _ => todo!(),
    //     }
    // }
}
