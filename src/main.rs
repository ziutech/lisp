use std::{
    collections::HashMap,
    error,
    fmt::Display,
    io::{stdin, stdout, BufRead, Write},
};

#[derive(Debug)]
enum Expr {
    Number(i32),
    String(String),
    Ident {
        ident: String,
        is_bind: bool,
    },
    Call {
        func_name: String,
        arguments: Vec<Expr>,
    },
}

#[derive(Debug, Clone)]
enum Value {
    Number(i32),
    String(String),
    Func(Func),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\t")?;
        match self {
            Value::Number(i) => writeln!(f, "{i}"),
            Value::String(s) => writeln!(f, "\"{s}\""),
            Value::Func(_) => todo!("display for functions"),
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
    At,
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
            b'@' => {
                self.position += 1;
                Some(Token {
                    token_type: TokenType::At,
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
            b' ' => {
                self.position += 1;
                self.make_token()
            }
            b'\n' => return None,
            a => panic!("unexpected token: {}", a),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.make_token()
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
        let tok = self.tokens.next().unwrap().token_type;
        if tok != TokenType::LeftParen {
            panic!("expected left paren, got: {tok:?}");
        }
        self.parse_sexpr()
    }

    fn parse_sexpr(&mut self) -> Expr {
        let TokenType::Ident(func_name) = self.tokens.next().unwrap().token_type else {
            panic!("expected an identifier");
        };
        let mut arguments = Vec::new();
        loop {
            let tok = self.tokens.next().unwrap();
            match tok.token_type {
                TokenType::LeftParen => {
                    let expr = self.parse_sexpr();
                    arguments.push(expr);
                }
                TokenType::String(s) => arguments.push(Expr::String(s)),
                TokenType::Ident(s) => arguments.push(Expr::Ident {
                    ident: s,
                    is_bind: false,
                }),
                TokenType::Number(i) => arguments.push(Expr::Number(i)),
                TokenType::RightParen => break,
                TokenType::At => {
                    let ident = match self.tokens.next().unwrap().token_type {
                        TokenType::Ident(s) => s,
                        t => panic!("invalid token: {t:#?}"),
                    };
                    arguments.push(Expr::Ident {
                        ident,
                        is_bind: true,
                    });
                }
            }
        }
        Expr::Call {
            func_name,
            arguments,
        }
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

fn r#let(args: &[Value], env: &mut Env<'_>) -> Value {
    let name = args[0].as_str();
    let value = args[1].clone();
    env.insert(name.to_owned(), value.clone());
    value.clone()
}

fn id(args: &[Value], env: &mut Env<'_>) -> Value {
    args[0].clone()
}

// TODO: add manually triggered garbage collector
fn eval(expr: &Expr, env: &mut Env<'_>) -> Value {
    match expr {
        Expr::String(s) => Value::String(s.clone()),
        Expr::Number(i) => Value::Number(*i),
        Expr::Ident { ident, is_bind } => {
            if *is_bind {
                Value::String(ident.clone())
            } else {
                env.get(ident).expect("undefined identifier").clone()
            }
        }
        Expr::Call {
            func_name,
            arguments,
        } => {
            let arguments: Vec<Value> = arguments.iter().map(|expr| eval(expr, env)).collect();
            let func = env
                .get(func_name.as_str())
                .expect("undefined function")
                .as_func();

            func(&arguments, env)
        }
    }
}

type Func = fn(&[Value], env: &mut Env<'_>) -> Value;
type Env<'a> = HashMap<String, Value>;

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut env = HashMap::new();
    env.insert("plus".to_owned(), Value::Func(plus));
    env.insert("let".to_owned(), Value::Func(r#let));
    env.insert("id".to_owned(), Value::Func(id));
    loop {
        print!("repl> ");
        stdout().lock().flush()?;
        let mut buf = String::new();
        stdin().lock().read_line(&mut buf)?;
        let text: Vec<u8> = buf.bytes().collect();
        let expr = Parser::new(&text).parse();
        println!("{expr:?}");

        let result = eval(&expr, &mut env);
        println!("{}", result);
    }
}
