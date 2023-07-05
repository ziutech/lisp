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
        NotAValidName,
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                Error::UndefinedCall => write!(f, "Invalid call object")?,
                Error::NotEnoughParameters => write!(f, "Not enough parameters")?,
                Error::EmptyCall => write!(f, "Empty call")?,
                Error::NotAValidName => write!(f, "Invalid reference")?,
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
    fn call(&self, args: &[ast::Expr]) -> Result<ast::Expr, misc::Error>;
}

#[derive(Clone)]
pub struct BuiltinFunction {
    f: Func,
}

impl Callable for BuiltinFunction {
    fn call(&self, params: &[ast::Expr]) -> Result<ast::Expr, misc::Error> {
        Ok((self.f)(params))
    }
}

#[derive(Clone)]
enum Value {
    F(BuiltinFunction),
    E(ast::Expr),
}

#[derive(Clone)]
struct Environment {
    outer: Option<usize>,
    definitions: HashMap<String, Value>,
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
        let definitions = HashMap::from([
            ("+".to_owned(), Value::F(builtins::add())),
            ("*".to_owned(), Value::F(builtins::mul())),
            ("-".to_owned(), Value::F(builtins::sub())),
            ("/".to_owned(), Value::F(builtins::div())),
        ]);
        Environment {
            outer: None,
            definitions,
        }
    }

    fn new(outer_env_id: usize) -> Self {
        Environment {
            outer: Some(outer_env_id),
            definitions: Default::default(),
        }
    }

    fn add_definition(&mut self, name: &str, value: ast::Expr) -> Box<&Value> {
        use std::collections::hash_map::Entry::*;
        match self.definitions.entry(name.to_string()) {
            Occupied(mut o) => {
                o.insert(Value::E(value));
                Box::new(o.into_mut())
            }
            Vacant(v) => Box::new(v.insert(Value::E(value))),
        }
    }

    fn get_definition<'a, 'b>(
        current_env_id: usize,
        env: &'a [Environment],
        name: &'b str,
    ) -> Option<Box<&'a Value>> {
        let current = &env[current_env_id];
        if let Some(def) = current.definitions.get(name) {
            return Some(Box::new(def));
        }
        match current.outer {
            Some(outer) => Self::get_definition(outer, env, name),
            None => None,
        }
    }
}

struct State {
    environments: Vec<Environment>,
}

fn get_value<'a>(
    state: &'a mut State,
    name: &ast::Expr,
    args: &[ast::Expr],
) -> Result<Box<&'a Value>, misc::Error> {
    use ast::Expr::*;
    match name {
        Ident(ref keyword) if keyword == "let" => {
            let Ident(variable_name) = &args[0] else {
                return Err(misc::Error::NotAValidName);
            };
            let value = {
                let env = &mut state.environments;
                env.last_mut()
                    .unwrap()
                    .add_definition(variable_name, args[1].clone())
            };
            Ok(value)
        }
        Ident(name) => {
            let last = state.environments.len() - 1;
            Environment::get_definition(last, &state.environments, name)
        }
        .ok_or(misc::Error::UndefinedCall),
        Add | Sub | Mul | Div => {
            let last = state.environments.len() - 1;
            Environment::get_definition(last, &state.environments, &name.to_string())
        }
        .ok_or(misc::Error::UndefinedCall),
        _ => return Err(misc::Error::UndefinedCall),
    }
}

fn read(str: &str) -> ast::Expr {
    parse::parser().parse(str).unwrap()
}

fn eval(state: &mut State, expr: &ast::Expr) -> Result<ast::Expr, misc::Error> {
    use ast::Expr::*;
    let new_env_id: usize = state.environments.len();
    let env = Environment::new(new_env_id);
    state.environments.push(env);
    if let List(args) = expr {
        let name = args.get(0).ok_or(misc::Error::EmptyCall)?;
        println!("{:?}", &args[1..]);
        let args = &args[1..];
        let mut params = Vec::with_capacity(args.len());
        for arg in args {
            params.push(eval(state, arg)?);
        }
        let value = get_value(state, name, &args[1..])?;
        let func = match &**value {
            Value::E(expr) => return Ok(expr.clone()),
            Value::F(f) => f,
        };
        func.call(&params)
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
    let ast = read(str);
    let mut state = {
        let env = Environment::standard();
        let mut environments = Vec::with_capacity(8);
        environments.push(env);
        State { environments }
    };
    let result = eval(&mut state, &ast);
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
