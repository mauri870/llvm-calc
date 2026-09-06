use pest::Parser;
use pest::iterators::Pairs;
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest_derive::Parser;

use crate::ast::{BinOp, Expr};

#[derive(Parser)]
#[grammar = "src/calc.pest"]
struct CalcParser;

pub fn parse(input: &str) -> Result<Expr, String> {
    let mut pairs = CalcParser::parse(Rule::calculation, input)
        .map_err(|e| e.to_string())?;
    let expr_pair = pairs.next().unwrap();
    Ok(build_expr(expr_pair.into_inner()))
}

fn build_expr(pairs: Pairs<Rule>) -> Expr {
    let pratt = PrattParser::new()
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left));

    pratt
        .map_primary(|primary| match primary.as_rule() {
            Rule::number => Expr::Number(primary.as_str().parse().unwrap()),
            // if parenthesised expr, recurse into inner expr pairs
            Rule::expr => build_expr(primary.into_inner()),
            rule => unreachable!("unexpected primary rule: {rule:?}"),
        })
        .map_infix(|left, op, right| Expr::BinOp {
            op: match op.as_rule() {
                Rule::add => BinOp::Add,
                Rule::sub => BinOp::Sub,
                Rule::mul => BinOp::Mul,
                Rule::div => BinOp::Div,
                rule => unreachable!("unexpected infix rule: {rule:?}"),
            },
            left: Box::new(left),
            right: Box::new(right),
        })
        .parse(pairs)
}
