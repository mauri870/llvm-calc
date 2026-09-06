#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    BinOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
}
