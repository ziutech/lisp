use std::collections::HashMap;
use std::error::Error;
use std::io::{BufRead, Write};

use std::result::Result;

use chumsky::Parser;

mod misc {
    pub enum Error {
        UndefinedCall,
        NotEnoughParameters,
        EmptyCall,
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                Error::UndefinedCall => write!(f, "Invalid call object")?,
                Error::NotEnoughParameters => write!(f, "Not enough parameters")?,
                Error::EmptyCall => write!(f, "Empty call")?,
            };
            Ok(())
        }
    }
}

mod ast;
mod parse;

//TODO: move this to separate modules
type Func = fn(&[ast::Expr]) -> ast::Expr;

trait Callable {
    fn call(&self, env: &Environment, args: &[ast::Expr]) -> Result<ast::Expr, misc::Error>;
}

pub struct BuiltinFunction {
    f: Func,
}

impl Callable for BuiltinFunction {
    fn call(&self, env: &Environment, args: &[ast::Expr]) -> Result<ast::Expr, misc::Error> {
        let params: &Vec<ast::Expr> = &args
            .iter()
            .map(|exp| eval(env, exp))
            .collect::<Result<Vec<ast::Expr>, misc::Error>>()?;
        Ok((self.f)(params))
    }
}

struct Environment {
    functions: HashMap<String, BuiltinFunction>,
}

// TODO: Macro for defining BuiltinFunction
mod builtins {
    use crate::{ast, BuiltinFunction};
    fn _add(args: &[ast::Expr]) -> ast::Expr {
        args.iter().sum()
    }

    pub fn add() -> BuiltinFunction {
        BuiltinFunction { f: _add }
    }

    fn _sub(args: &[ast::Expr]) -> ast::Expr {
        &args[0] - &_add(&args[1..])
    }

    pub fn sub() -> BuiltinFunction {
        BuiltinFunction { f: _sub }
    }

    fn _mul(args: &[ast::Expr]) -> ast::Expr {
        args.iter().product()
    }

    pub fn mul() -> BuiltinFunction {
        BuiltinFunction { f: _mul }
    }

    fn _div(args: &[ast::Expr]) -> ast::Expr {
        &args[0] / &_mul(&args[1..])
    }

    pub fn div() -> BuiltinFunction {
        BuiltinFunction { f: _div }
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

    fn get_fn(&self, name: &str) -> Option<&BuiltinFunction> {
        self.functions.get(name)
    }
}

fn read(str: &str) -> ast::Expr {
    parse::parser().parse(str).unwrap()
}

fn eval(env: &Environment, expr: &ast::Expr) -> Result<ast::Expr, misc::Error> {
    use ast::Expr::*;
    if let List(args) = expr {
        let name = args.get(0).ok_or(misc::Error::EmptyCall)?;
        println!("{:?}", &args[1..]);
        match name {
            Ident(name) => env.get_fn(name),
            Add | Sub | Mul | Div => env.get_fn(&name.to_string()),
            _ => return Err(misc::Error::UndefinedCall),
        }
        .ok_or(misc::Error::UndefinedCall)?
        .call(env, &args[1..])
    } else {
        // FIXME: return &expr because vectors and hashmaps (or boxed? (probably boxed...))
        Ok(expr.clone())
    }
}

fn print(res: Result<ast::Expr, misc::Error>) -> String {
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
