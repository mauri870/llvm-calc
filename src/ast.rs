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
    Var(String),
    BinOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub bindings: Vec<(String, Expr)>,
    pub body: Expr,
}
