use chumsky::prelude::*;

use crate::ast::Expr;

pub fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    let whitespace = || one_of(", ").repeated();
    recursive(|expr| {
        let int = text::digits(10)
            .map(|s: String| Expr::Num(s.parse().unwrap()))
            .padded_by(whitespace());

        let ident = text::ident().map(Expr::Ident);
        let symbol = choice((
            just("+").to(Expr::Add),
            just("-").to(Expr::Sub),
            just("*").to(Expr::Mul),
            just("/").to(Expr::Div),
        ))
        .padded_by(whitespace());
        let atom = int
            .or(text::keyword("nil").map(|_| Expr::Nil))
            .or(text::keyword("true").map(|_| Expr::True))
            .or(text::keyword("false").map(|_| Expr::False))
            .or(ident)
            .or(symbol);
        let list = expr
            .clone()
            .padded_by(whitespace())
            .repeated()
            .or_not()
            .map(|o| match o {
                Some(v) => Expr::List(v),
                None => Expr::List(vec![]),
            })
            .delimited_by(
                just('(').padded_by(whitespace()),
                just(')').padded_by(whitespace()),
            );

        list.or(atom)
    })
    .then_ignore(just('\n'))
}

#[cfg(test)]
mod tests {
    use crate::ast::Expr::*;
    use crate::parse::*;

    fn parse_single_expr(s: &str) -> Expr {
        parser().parse(s.to_owned() + "\n").unwrap()
    }

    #[test]
    fn numbers() {
        assert_eq!(parse_single_expr("52"), Expr::Num(52));
    }
    #[test]
    fn symbols() {
        assert_eq!(parse_single_expr("+"), Add);
        assert_eq!(parse_single_expr("-"), Sub);
        assert_eq!(parse_single_expr("*"), Mul);
        assert_eq!(parse_single_expr("/"), Div);
    }
    #[test]
    fn ident() {
        fn test_ident(s: &str) {
            assert_eq!(parse_single_expr(s), Expr::Ident(s.to_string()), "{}", s);
        }
        test_ident("abc5");
        test_ident("x_abcd");
    }
    #[test]
    fn lists() {
        assert_eq!(
            parse_single_expr("(+ 1 2)"),
            List(vec![Add, Num(1), Num(2)])
        );
        assert_eq!(parse_single_expr("()"), List(Vec::new()));
        assert_eq!(parse_single_expr("(  )"), List(Vec::new()));
        assert_eq!(
            parse_single_expr("( +   1   (-   2 3   )   )  "),
            List(vec![Add, Num(1), List(vec![Sub, Num(2), Num(3)])])
        );
        assert_eq!(
            parse_single_expr("(()())"),
            List(vec![List(vec![]), List(vec![])])
        );
        assert_eq!(
            parse_single_expr("(1 2, 3,,,,),,"),
            List(vec![Num(1), Num(2), Num(3)])
        );
    }

    #[test]
    fn builtins() {
        assert_eq!(parse_single_expr("nil"), Nil);
        assert_eq!(parse_single_expr("true"), True);
        assert_eq!(parse_single_expr("false"), False);
    }
}
