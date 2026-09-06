use crate::ast::{BinOp, Expr};

pub fn parse(input: &str) -> Result<Expr, String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    match tokens.as_slice() {
        [n] => parse_number(n),
        [left, op, right] => Ok(Expr::BinOp {
            op: parse_op(op)?,
            left: Box::new(parse_number(left)?),
            right: Box::new(parse_number(right)?),
        }),
        _ => Err(format!("expected '<num>' or '<num> <op> <num>', got: {input:?}")),
    }
}

fn parse_number(s: &str) -> Result<Expr, String> {
    s.parse::<f64>()
        .map(Expr::Number)
        .map_err(|_| format!("not a number: {s:?}"))
}

fn parse_op(s: &str) -> Result<BinOp, String> {
    match s {
        "+" => Ok(BinOp::Add),
        "-" => Ok(BinOp::Sub),
        "*" => Ok(BinOp::Mul),
        "/" => Ok(BinOp::Div),
        _ => Err(format!("unknown operator: {s:?}")),
    }
}
