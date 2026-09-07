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
pub enum Cond {
    Cmp { op: CmpOp, left: Box<Expr>, right: Box<Expr> },
    And(Box<Cond>, Box<Cond>),
    Or(Box<Cond>, Box<Cond>),
    Not(Box<Cond>),
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
    Call { name: String, args: Vec<Expr> },
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<FnDef>,
    pub bindings: Vec<(String, Expr)>,
    pub body: Expr,
}
