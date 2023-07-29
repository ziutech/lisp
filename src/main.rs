use std::{
    borrow::Borrow,
    collections::HashMap,
    error,
    fmt::Display,
    io::{stdin, stdout, BufRead, Write},
};

use std::hash::Hash;

#[derive(Debug, Clone)]
enum Expr {
    Number(i32),
    String(String),
    Ident {
        ident: String,
    },
    Array(Vec<Expr>),
    // TODO: order should be name -> is_macro -> arguments
    Call {
        func_name: String,
        arguments: Vec<Expr>,
        is_macro: bool,
    },
}

impl Expr {
    fn as_ident(&self) -> &str {
        match self {
            Expr::Ident { ident } => ident,
            _ => panic!("not an ident"),
        }
    }
}

#[derive(Debug, Clone)]
enum Value {
    Nil,
    Number(i32),
    String(String),
    Array(Vec<Value>),
    Func(Func),
    Macro(Macro),
    DefFunc(DefFunc),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Number(i) => write!(f, "{i}"),
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Func(f) => todo!("func: {f:?}"),
            Value::Macro(f) => todo!("macro {f:?}"),
            Value::Array(values) => {
                write!(f, "[")?;
                for v in values {
                    write!(f, " {}", v)?;
                }
                writeln!(f, " ]")?;
                Ok(())
            }
            Value::DefFunc(f) => todo!("{f}"),
        }
    }
}

impl Value {
    fn as_func(&self) -> &Func {
        match self {
            Value::Func(f) => f,
            _ => panic!("not a function"),
        }
    }

    fn as_macro(&self) -> &Macro {
        match self {
            Value::Macro(f) => f,
            _ => panic!("not a macro"),
        }
    }

    fn as_str<'a>(&'a self) -> &'a str {
        match self {
            Value::String(s) => s.as_str(),
            _ => panic!("not a str"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum TokenType {
    LeftParen,
    RightParen,
    LeftSquare,
    RightSquare,
    Colon,
    String(String),
    Ident(String),
    Number(i32),
}

#[derive(Debug)]
struct Token {
    token_type: TokenType,
}

struct Lexer<'a> {
    text: &'a [u8],
    position: usize,
    second_position: usize,
}

impl<'a> Lexer<'a> {
    fn new(text: &'a [u8]) -> Self {
        Self {
            text,
            position: 0,
            second_position: 0,
        }
    }

    fn make_token(&mut self) -> Option<Token> {
        let mut position = self.position;
        let c = self.text.get(position)?;

        match c {
            b'(' => {
                self.position += 1;
                Some(Token {
                    token_type: TokenType::LeftParen,
                })
            }
            b')' => {
                self.position += 1;
                Some(Token {
                    token_type: TokenType::RightParen,
                })
            }
            b'[' => {
                self.position += 1;
                Some(Token {
                    token_type: TokenType::LeftSquare,
                })
            }
            b']' => {
                self.position += 1;
                Some(Token {
                    token_type: TokenType::RightSquare,
                })
            }
            b':' => {
                self.position += 1;
                Some(Token {
                    token_type: TokenType::Colon,
                })
            }
            b'"' => {
                position += 1;
                loop {
                    let c = self.text.get(position)?;
                    match c {
                        b'"' => {
                            // self.pos + 1 to remove first quotes
                            let string =
                                std::str::from_utf8(&self.text[(self.position + 1)..position])
                                    .unwrap()
                                    .to_owned();
                            self.position = position + 1; // + 1 to skip last quote
                            break Some(Token {
                                token_type: TokenType::String(string),
                            });
                        }
                        _ => position += 1,
                    }
                }
            }
            b'0'..=b'9' => loop {
                let c = self.text.get(position)?;
                match c {
                    b'0'..=b'9' => position += 1,
                    _ => {
                        let num: i32 = std::str::from_utf8(&self.text[self.position..position])
                            .unwrap()
                            .parse()
                            .unwrap();
                        self.position = position;
                        break Some(Token {
                            token_type: TokenType::Number(num),
                        });
                    }
                }
            },
            b'a'..=b'z' | b'A'..=b'Z' => loop {
                let c = self.text.get(position)?;
                match c {
                    b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => position += 1,
                    _ => {
                        let ident = std::str::from_utf8(&self.text[self.position..position])
                            .unwrap()
                            .to_owned();
                        self.position = position;
                        break Some(Token {
                            token_type: TokenType::Ident(ident),
                        });
                    }
                }
            },
            b' ' | b'\t' | b'\n' => {
                self.position += 1;
                self.make_token()
            }
            a => panic!("unexpected token: {}", a),
        }
    }
}

struct Parser<'a> {
    tokens: Lexer<'a>,
}

impl<'a> Parser<'a> {
    fn new(text: &'a [u8]) -> Self {
        Parser {
            tokens: Lexer::new(text),
        }
    }

    pub fn parse(&mut self) -> Expr {
        let tok = self.tokens.make_token().unwrap().token_type;
        if tok != TokenType::LeftParen {
            panic!("expected left paren, got: {tok:?}");
        }
        self.parse_sexpr()
    }

    fn parse_sexpr(&mut self) -> Expr {
        let mut is_macro = false;
        let func_name = match self.tokens.make_token().unwrap().token_type {
            TokenType::Colon => {
                is_macro = true;
                let TokenType::Ident(ident) = self.tokens.make_token().unwrap().token_type else {
                    panic!("expected an colon or identifier");
                };
                ident
            }
            TokenType::Ident(ident) => ident,
            _ => panic!("expected an colon or identifier"),
        };
        let mut arguments = Vec::new();
        loop {
            let tok = self.tokens.make_token().unwrap();
            match tok.token_type {
                TokenType::LeftParen => {
                    let expr = self.parse_sexpr();
                    arguments.push(expr);
                }
                TokenType::String(s) => arguments.push(Expr::String(s)),
                TokenType::Ident(s) => arguments.push(Expr::Ident { ident: s }),
                TokenType::Number(i) => arguments.push(Expr::Number(i)),
                TokenType::RightParen => break,
                TokenType::Colon => panic!("unexpected colon"),
                TokenType::LeftSquare => arguments.push(self.parse_array()),
                TokenType::RightSquare => panic!("unexpected right square bracket"),
            }
        }
        Expr::Call {
            func_name,
            arguments,
            is_macro,
        }
    }

    fn parse_array(&mut self) -> Expr {
        let mut elements = Vec::new();
        loop {
            let tok = self.tokens.make_token().unwrap();
            match tok.token_type {
                TokenType::String(s) => elements.push(Expr::String(s)),
                TokenType::Ident(i) => elements.push(Expr::Ident { ident: i }),
                TokenType::Number(n) => elements.push(Expr::Number(n)),
                TokenType::RightSquare => break,
                _ => panic!("unexpected {:?}", tok),
            }
        }
        Expr::Array(elements)
    }
}

fn plus(args: &[Value], env: &mut Env<'_>) -> Value {
    let mut acc = 0;
    for v in args.iter() {
        match v {
            Value::Number(a) => acc += a,
            _ => todo!(),
        }
    }
    return Value::Number(acc);
}

fn r#let(args: &[Expr], env: &mut Env<'_>) -> Value {
    let name = args[0].as_ident();
    let value = eval(&args[1], env);
    env.insert(name.to_owned(), value.clone());
    value.clone()
}

#[derive(Debug, Clone)]
struct DefFunc {
    params: Vec<String>,
    code: Expr,
}

impl Display for DefFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "args: {:?}", self.params)?;
        write!(f, "code: \n{:?}", self.code)?;
        Ok(())
    }
}

impl DefFunc {
    fn call(&self, args: &[Value], env: &mut Env<'_>) -> Value {
        let mut new_env = Env::default();
        for (i, p) in self.params.iter().enumerate() {
            new_env.insert(p.to_owned(), args[i].clone());
        }
        new_env.add_outer(env);
        eval(&self.code, &mut new_env)
    }
}

fn def(args: &[Expr], env: &mut Env<'_>) -> Value {
    let name = args[0].as_ident();
    let last_index = args.len() - 1;
    let params = args[1..(last_index)]
        .iter()
        .map(|x| x.as_ident().to_owned())
        .collect();
    let code = args.last().unwrap().clone();
    let def_func = DefFunc { params, code };
    let def_func = Value::DefFunc(def_func);
    env.insert(name.to_owned(), def_func.clone());
    Value::Nil
}

fn id(args: &[Value], env: &mut Env<'_>) -> Value {
    args[0].clone()
}

fn scope(exprs: &[Expr], env: &mut Env<'_>) -> Value {
    let mut new_env = Env::default();
    new_env.add_outer(env);
    new_env.insert(
        "print".to_owned(),
        Value::Func(|v, _| {
            println!("{v:?}");
            v[0].clone()
        }),
    );
    let mut some_value = None;
    for expr in exprs {
        some_value = Some(eval(expr, &mut new_env));
    }
    some_value.unwrap_or(Value::Nil)
}

// TODO: add manually triggered garbage collector
fn eval(expr: &Expr, env: &mut Env<'_>) -> Value {
    match expr {
        Expr::String(s) => Value::String(s.clone()),
        Expr::Number(i) => Value::Number(*i),
        Expr::Ident { ident } => env.get(ident).expect("undefined identifier").clone(),
        Expr::Call {
            func_name,
            arguments,
            is_macro,
        } => {
            if !is_macro {
                let evaled_arguments: Vec<Value> =
                    arguments.iter().map(|expr| eval(expr, env)).collect();
                let func = env
                    .get(func_name.as_str())
                    .expect(&format!("undefined value: {}", func_name))
                    .clone();
                match func {
                    Value::Func(f) => f(&evaled_arguments, env),
                    Value::DefFunc(f) => f.call(&evaled_arguments, env),
                    _ => panic!("not a function"),
                }
            } else {
                let func = env
                    .get(func_name.as_str())
                    .expect(&format!("undefined value: {}", func_name))
                    .as_macro();
                func(arguments, env)
            }
        }
        Expr::Array(exprs) => {
            let mut values = vec![];
            for expr in exprs {
                values.push(eval(expr, env));
            }
            Value::Array(values)
        }
    }
}

type Func = fn(&[Value], env: &mut Env<'_>) -> Value;
type Macro = fn(&[Expr], env: &mut Env<'_>) -> Value;

#[derive(Default)]
struct Env<'a> {
    outer: Option<&'a Env<'a>>,
    defs: HashMap<String, Value>,
}

impl<'a> Env<'a> {
    fn add_outer(&mut self, env: &'a mut Env<'_>) {
        self.outer.replace(env);
    }
    fn insert(&mut self, id: String, value: Value) -> Option<Value> {
        self.defs.insert(id, value)
    }
    fn get<Q>(&self, id: &Q) -> Option<&Value>
    where
        Q: Hash + Eq + ?Sized,
        String: Borrow<Q>,
    {
        match self.defs.get(id) {
            Some(v) => Some(v),
            None => match self.outer {
                Some(o) => o.get(id),
                None => None,
            },
        }
    }

    fn print_defs(&self) {
        for x in &self.defs {
            println!("{x:?}");
        }
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let debug = std::env::args()
        .skip(1)
        .take(1)
        .next()
        .map(|x| x.parse().unwrap())
        .unwrap_or(false);
    let mut env = Env::default();
    env.insert("plus".to_owned(), Value::Func(plus));
    env.insert("let".to_owned(), Value::Macro(r#let));
    env.insert("id".to_owned(), Value::Func(id));
    env.insert("scope".to_owned(), Value::Macro(scope));
    env.insert("def".to_owned(), Value::Macro(def));
    loop {
        print!(":: ");
        stdout().lock().flush()?;
        let mut nestion = 0; // how deep is the code nested / how many unclosed '(' there are
        let mut text = vec![];
        loop {
            let mut buf = String::new();
            stdin().lock().read_line(&mut buf)?;
            for x in buf.bytes() {
                match x {
                    b'(' => nestion += 1,
                    b')' => nestion -= 1,
                    _ => {}
                }
            }
            text.extend_from_slice(buf.as_bytes());
            if nestion == 0 {
                break;
            }
            print!("-- ");
            for _ in 0..nestion {
                print!("  ");
            }
            stdout().flush();
        }
        let expr = Parser::new(&text).parse();
        if debug {
            println!("{expr:?}");
        }

        let result = eval(&expr, &mut env);
        println!("== {}", result);
    }
}
