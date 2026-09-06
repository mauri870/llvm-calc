#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum CmpOp {
    Lt,
    Gt,
    Eq,
    Ne,
    Le,
    Ge,
}

#[derive(Debug, Clone)]
pub struct Cond {
    pub op: CmpOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub bindings: Vec<(String, Expr)>,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    Var(String),
    Neg(Box<Expr>),
    BinOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    If { cond: Box<Cond>, then: Box<Expr>, else_: Box<Expr> },
    While { cond: Box<Cond>, body: Box<Block> },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub bindings: Vec<(String, Expr)>,
    pub body: Expr,
}
