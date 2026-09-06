use pest::Parser;
use pest::iterators::Pairs;
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest_derive::Parser;

use crate::ast::{BinOp, Block, CmpOp, Cond, Expr, Program};

#[derive(Parser)]
#[grammar = "src/calc.pest"]
struct CalcParser;

pub fn parse(input: &str) -> Result<Program, String> {
    let pairs = CalcParser::parse(Rule::program, input)
        .map_err(|e| e.to_string())?;

    let mut bindings = Vec::new();
    let mut body = None;

    for pair in pairs {
        match pair.as_rule() {
            Rule::assign => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let expr_pair = inner.next().unwrap();
                bindings.push((name, build_expr(expr_pair.into_inner())));
            }
            Rule::expr => {
                body = Some(build_expr(pair.into_inner()));
            }
            Rule::EOI => {}
            rule => unreachable!("unexpected top-level rule: {rule:?}"),
        }
    }

    Ok(Program { bindings, body: body.unwrap() })
}

fn build_expr(pairs: Pairs<Rule>) -> Expr {
    let pratt = PrattParser::new()
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left));

    pratt
        .map_primary(|primary| match primary.as_rule() {
            Rule::number   => Expr::Number(primary.as_str().parse().unwrap()),
            Rule::ident    => Expr::Var(primary.as_str().to_string()),
            Rule::neg      => Expr::Neg(Box::new(build_expr(primary.into_inner()))),
            Rule::expr     => build_expr(primary.into_inner()),
            Rule::if_expr  => build_if(primary.into_inner()),
            Rule::while_expr => build_while(primary.into_inner()),
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

fn build_if(mut pairs: Pairs<Rule>) -> Expr {
    let cond_pair = pairs.next().unwrap();
    let then_pair = pairs.next().unwrap();
    let else_pair = pairs.next().unwrap();
    Expr::If {
        cond: Box::new(build_cond(cond_pair.into_inner())),
        then: Box::new(build_expr(then_pair.into_inner())),
        else_: Box::new(build_expr(else_pair.into_inner())),
    }
}

fn build_while(mut pairs: Pairs<Rule>) -> Expr {
    let cond_pair = pairs.next().unwrap();  // Rule::cond
    let block_pair = pairs.next().unwrap(); // Rule::block
    Expr::While {
        cond: Box::new(build_cond(cond_pair.into_inner())),
        body: Box::new(build_block(block_pair.into_inner())),
    }
}

fn build_block(pairs: Pairs<Rule>) -> Block {
    let mut bindings = Vec::new();
    let mut body = None;
    for pair in pairs {
        match pair.as_rule() {
            Rule::assign => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let expr_pair = inner.next().unwrap();
                bindings.push((name, build_expr(expr_pair.into_inner())));
            }
            Rule::expr => {
                body = Some(build_expr(pair.into_inner()));
            }
            rule => unreachable!("unexpected block rule: {rule:?}"),
        }
    }
    Block { bindings, body: Box::new(body.unwrap()) }
}

fn build_cond(mut pairs: Pairs<Rule>) -> Cond {
    let left_pair  = pairs.next().unwrap();
    let op_pair    = pairs.next().unwrap();
    let right_pair = pairs.next().unwrap();
    Cond {
        op: match op_pair.as_rule() {
            Rule::lt => CmpOp::Lt,
            Rule::gt => CmpOp::Gt,
            Rule::eq => CmpOp::Eq,
            Rule::ne => CmpOp::Ne,
            Rule::le => CmpOp::Le,
            Rule::ge => CmpOp::Ge,
            rule => unreachable!("unexpected cmpop rule: {rule:?}"),
        },
        left:  Box::new(build_expr(left_pair.into_inner())),
        right: Box::new(build_expr(right_pair.into_inner())),
    }
}
