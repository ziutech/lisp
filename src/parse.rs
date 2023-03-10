use chumsky::prelude::*;

use crate::ast::Expr;

pub fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    let whitespace = || one_of(", ").repeated();
    recursive(|expr| {
        let int = text::digits(10)
            .map(|s: String| Expr::Num(s.parse().unwrap()))
            .padded_by(whitespace());

        let ident = text::ident().map(|s| Expr::Ident(s));
        let symbol = move |c: &'static str| just(c).to(Expr::Ident(c.to_string()));
        let symbol =
            choice((symbol("+"), symbol("-"), symbol("*"), symbol("/"))).padded_by(whitespace());
        let atom = int.or(ident).or(symbol);
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
            parse_single_expr("(nil)"),
            List(vec![Ident("nil".to_string())])
        );
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
}