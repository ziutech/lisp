use std::{
    error,
    io::{stdin, stdout, BufRead, Write},
};

#[derive(Debug)]
enum Expr {
    Number(i32),
    Ident(String),
    Call {
        func_name: String,
        arguments: Vec<Expr>,
    },
}

#[derive(Debug, Eq, PartialEq)]
enum TokenType {
    LeftParen,
    RightParen,
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
                TokenType::Ident(s) => arguments.push(Expr::Ident(s)),
                TokenType::Number(i) => arguments.push(Expr::Number(i)),
                TokenType::RightParen => break,
            }
        }
        Expr::Call {
            func_name,
            arguments,
        }
    }
}

fn eval(expr: &Expr) {}

fn main() -> Result<(), Box<dyn error::Error>> {
    loop {
        print!("repl> ");
        stdout().lock().flush()?;
        let mut buf = String::new();
        stdin().lock().read_line(&mut buf)?;
        let text: Vec<u8> = buf.bytes().collect();
        let expr = Parser::new(&text).parse();
        println!("{:?}", expr);
    }
}
