use std::collections::HashMap;
use std::error::Error;
use std::io::{BufRead, Write};

use std::result::Result;

use chumsky::Parser;

mod ast;
mod parse;

type Func = fn(&[ast::Expr]) -> ast::Expr;

pub struct Function {
    arity: u8,
    infinite: bool,
    f: Func,
}

impl std::ops::Deref for Function {
    type Target = Func;

    fn deref(&self) -> &Self::Target {
        &self.f
    }
}

struct Environment {
    functions: HashMap<String, Function>,
}

mod builtins {
    use crate::{ast, Function};
    fn _add(args: &[ast::Expr]) -> ast::Expr {
        args.into_iter().sum()
    }

    pub fn add() -> Function {
        Function {
            arity: u8::MAX,
            infinite: true,
            f: _add,
        }
    }

    fn _sub(args: &[ast::Expr]) -> ast::Expr {
        match (args.get(0), args.get(1)) {
            (Some(a), Some(b)) => a - b,
            _ => todo!("Resolving not enough arguments in `(- ...)`"),
        }
    }

    pub fn sub() -> Function {
        Function {
            arity: 2,
            infinite: false,
            f: _sub,
        }
    }

    fn _mul(args: &[ast::Expr]) -> ast::Expr {
        args.into_iter().product()
    }

    pub fn mul() -> Function {
        Function {
            arity: u8::MAX,
            infinite: true,
            f: _mul,
        }
    }

    fn _div(args: &[ast::Expr]) -> ast::Expr {
        match (args.get(0), args.get(1)) {
            (Some(a), Some(b)) => a / b,
            _ => todo!("Resolving not enough arguments in `(/ ...)`"),
        }
    }

    pub fn div() -> Function {
        Function {
            arity: 2,
            infinite: false,
            f: _div,
        }
    }
}

impl Environment {
    fn standard() -> Self {
        let functions = HashMap::from([
            ("+".to_owned(), builtins::add()),
            ("*".to_owned(), builtins::mul()),
            ("-".to_owned(), builtins::sub()),
            ("/".to_owned(), builtins::div()),
        ]);
        Environment { functions }
    }
}

fn read(str: &str) -> ast::Expr {
    parse::parser().parse(str).unwrap()
}

fn eval(env: &Environment, expr: &ast::Expr) -> Result<ast::Expr, &'static str> {
    use ast::Expr::*;
    if let List(args) = expr {
        let name = args.get(0).ok_or("No object to call")?;
        let func = match name {
            Ident(name) => env.functions.get(name),
            Add | Sub | Mul | Div => env.functions.get(&name.to_string()),
            _ => return Err("Invalid call object"),
        }
        .ok_or("Calling undeclared function")?;
        if !func.infinite && args.len() - 1 < func.arity.into() {
            return Err("Not enough parameters");
        }
        let params: &Vec<ast::Expr> = &args[1..]
            .into_iter()
            .map(|exp| eval(&env, exp))
            .collect::<Result<Vec<ast::Expr>, &str>>()?;
        Ok(func(params))
    } else {
        // FIXME: return &expr because vectors and hashmaps
        Ok(expr.clone())
    }
}

fn print(res: Result<ast::Expr, &str>) -> String {
    match res {
        Ok(expr) => format!("{}", expr),
        Err(e) => format!("ERROR: {}", e),
    }
}
fn rep(str: &str) -> String {
    let env = Environment::standard();
    let ast = read(str);
    let result = eval(&env, &ast);
    print(result)
}

fn _loop<R: BufRead, W: Write>(bufin: &mut R, bufout: &mut W) -> Result<(), Box<dyn Error>> {
    loop {
        // TODO: line editing
        // TODO: repl history
        write!(bufout, "user> ")?;
        bufout.flush()?;
        let mut input = String::new();
        if bufin.read_line(&mut input)? == 0 {
            break;
        }
        let output = rep(&input);
        writeln!(bufout, "{}", output)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: remove lock on stdin (?)
    let mut bufin = std::io::stdin().lock();
    let mut bufout = std::io::stdout();
    _loop(&mut bufin, &mut bufout)?;
    Ok(())
}

// #[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::io::Read;

    use crate::rep;
    use std::iter::zip;
    fn parse_test(str: &String) -> (String, String) {
        let mut input = String::new();
        let mut output = String::new();
        for line in str.lines() {
            match line.get(0..=0) {
                Some(";") => match line.get(1..=1) {
                    Some(";") => continue,
                    Some(">") => continue,
                    Some("=") => {
                        output.push_str(line.strip_prefix(";=>").unwrap());
                        output.push_str("\n");
                    }
                    Some(_) => continue,
                    None => continue,
                },
                Some(_) => {
                    input.push_str(line);
                    input.push_str("\n");
                }
                None => continue,
            }
        }
        (input, output)
    }

    // #[test]
    fn from_file() {
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap() + "/tests/step0_repl.mal";
        let mut file = File::open(dir).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let (input_str, expected_output_str) = parse_test(&contents);
        for (i, exp) in zip(input_str.lines(), expected_output_str.lines()) {
            assert_eq!(exp, rep(i));
        }
    }
}
