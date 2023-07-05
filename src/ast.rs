use std::{
    fmt::Display,
    iter::{Product, Sum},
    ops::{Add, Div, Mul, Sub},
};

// TODO: change to bytecode
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Expr {
    Num(f64),
    Symbol(String),
    Ident(String),
    Add,
    Sub,
    Mul,
    Div,
    Nil,
    True,
    False,

    // Neg,
    List(Vec<Expr>),
}

use Expr::*;

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Num(i) => write!(f, "{i}")?,
            Symbol(str) => write!(f, "{str}")?,
            Ident(ident) => write!(f, "{ident}")?,
            Add => write!(f, "+")?,
            Sub => write!(f, "-")?,
            Mul => write!(f, "*")?,
            Div => write!(f, "/")?,
            Nil => write!(f, "Nil")?,
            True => write!(f, "True")?,
            False => write!(f, "False")?,
            // Neg,
            List(l) => {
                write!(f, "({first}", first = l[0])?;
                for e in &l[1..] {
                    write!(f, " {e}")?;
                }
                write!(f, ")")?;
            }
        };
        Ok(())
    }
}

// TODO: use macro to generate traits for all possible
// combinations of references for std::ops::{Add, Sub, ...}
impl Add for Expr {
    type Output = Expr;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Num(a), Num(b)) => Num(a + b),
            _ => unimplemented!(),
        }
    }
}

impl<'a, 'b> Sub<&'b Expr> for &'a Expr {
    type Output = Expr;

    fn sub(self, rhs: &'b Expr) -> Self::Output {
        match (self, rhs) {
            (Num(a), Num(b)) => Num(a - b),
            _ => unimplemented!(),
        }
    }
}

impl Mul for Expr {
    type Output = Expr;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Num(a), Num(b)) => Num(a * b),
            _ => unimplemented!(),
        }
    }
}

// TODO: Divide by zero case
impl<'a, 'b> Div<&'b Expr> for &'a Expr {
    type Output = Expr;

    fn div(self, rhs: &'b Expr) -> Self::Output {
        match (self, rhs) {
            (Num(a), Num(b)) => Num(a / b),
            _ => unimplemented!(),
        }
    }
}

impl<'a> Sum<&'a Expr> for Expr {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
    {
        iter.fold(Num(0.0), |a, b| a + b.to_owned())
    }
}
impl<'a> Product<&'a Expr> for Expr {
    fn product<I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
    {
        iter.fold(Num(1.0), |a, b| a * b.to_owned())
    }
}
