use std::fmt::Display;
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]

pub enum Expr {
    Error,
    Num(i64),
    Symbol(String),
    Ident(String),
    Add,
    Sub,
    Mul,
    Div,

    // Neg,
    List(Vec<Expr>),
}

use Expr::*;

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error => write!(f, "ERROR")?,
            Num(i) => write!(f, "{i}")?,
            Symbol(str) => write!(f, "{str}")?,
            Ident(ident) => write!(f, "{ident}")?,
            Add => write!(f, "+")?,
            Sub => write!(f, "-")?,
            Mul => write!(f, "*")?,
            Div => write!(f, "/")?,
            // Neg,
            List(l) => {
                write!(f, "(")?;
                for e in l {
                    write!(f, "{e} ")?;
                }
                write!(f, ")")?;
            }
        };
        Ok(())
    }
}
