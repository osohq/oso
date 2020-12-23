use crate::lexer::Token;
use lalrpop_util::{lalrpop_mod, ParseError};

/// Used to denote whether an enclosed value is a value or a logical operator
pub enum ValueOrLogical {
    Value(Term),
    Logical(Term),
    Either(Term),
}

lalrpop_mod!(
    #[allow(clippy::all, dead_code, unused_imports, unused_mut)]
    polar
);

use super::error::{self, PolarResult};
use super::lexer::{self, Lexer};
use super::rules::*;
use super::terms::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Line {
    Rule(Rule),
    Query(Term),
}

fn to_parse_error(e: ParseError<usize, lexer::Token, error::ParseError>) -> error::ParseError {
    match e {
        ParseError::InvalidToken { location: loc } => error::ParseError::InvalidToken { loc },
        ParseError::UnrecognizedEOF { location: loc, .. } => {
            error::ParseError::UnrecognizedEOF { loc }
        }
        ParseError::UnrecognizedToken {
            token: (loc, t, _), ..
        } => match t {
            Token::Debug | Token::Cut | Token::In | Token::New => error::ParseError::ReservedWord {
                token: t.to_string(),
                loc,
            },
            _ => error::ParseError::UnrecognizedToken {
                token: t.to_string(),
                loc,
            },
        },
        ParseError::ExtraToken { token: (loc, t, _) } => error::ParseError::ExtraToken {
            token: t.to_string(),
            loc,
        },
        ParseError::User { error } => error,
    }
}

pub fn parse_term(src: &str) -> PolarResult<Term> {
    let mut errors = vec![];

    polar::TermParser::new()
        .parse(0, &mut errors, Lexer::new(src))
        .map_err(|e| to_parse_error(e).into())
        .and_then(|r| {
            if errors.is_empty() {
                Ok(r)
            } else {
                Err(to_parse_error(errors.remove(0).error).into())
            }
        })
}

pub fn parse_lines(src_id: u64, src: &str) -> PolarResult<Vec<Line>> {
    let mut errors = vec![];

    polar::LinesParser::new()
        .parse(src_id, &mut errors, Lexer::new(src))
        .map_err(|e| to_parse_error(e).into())
        .and_then(|r| {
            if errors.is_empty() {
                Ok(r)
            } else {
                Err(to_parse_error(errors.remove(0).error).into())
            }
        })
}

pub fn parse_query(src_id: u64, src: &str) -> PolarResult<Term> {
    let mut errors = vec![];
    polar::TermExpParser::new()
        .parse(src_id, &mut errors, Lexer::new(src))
        .map_err(|e| to_parse_error(e).into())
        .and_then(|r| {
            if errors.is_empty() {
                Ok(r)
            } else {
                Err(to_parse_error(errors.remove(0).error).into())
            }
        })
}

pub fn parse_query_with_errors(src_id: u64, src: &str) -> PolarResult<(Term, String)> {
    let mut errors = vec![];
    polar::TermExpParser::new()
        .parse(src_id, &mut errors, Lexer::new(src))
        .map_err(|e| to_parse_error(e).into())
        .map(|t| {
            (
                t,
                errors.into_iter().fold(String::new(), |acc, e| {
                    acc + &format!(
                        "{} at ({}, {}): {:?}",
                        to_parse_error(e.error).to_string(),
                        e.dropped_tokens[0].0,
                        e.dropped_tokens[0].2,
                        e.dropped_tokens[0].1
                    )
                }),
            )
        })
}

#[cfg(test)]
pub fn parse_rules(src_id: u64, src: &str) -> PolarResult<Vec<Rule>> {
    let mut errors = vec![];

    polar::RulesParser::new()
        .parse(src_id, &mut errors, Lexer::new(src))
        .map_err(|e| to_parse_error(e).into())
        .and_then(|r| {
            if errors.is_empty() {
                Ok(r)
            } else {
                Err(to_parse_error(errors.remove(0).error).into())
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatting::ToPolarString;
    use pretty_assertions::assert_eq;

    #[track_caller]
    fn parse_term(src: &str) -> Term {
        super::parse_term(src).unwrap()
    }

    #[track_caller]
    fn parse_query(src: &str) -> Term {
        super::parse_query(0, src).unwrap()
    }

    #[track_caller]
    fn parse_rule(src: &str) -> Rule {
        super::parse_rules(0, src).unwrap().pop().unwrap()
    }

    #[track_caller]
    fn parse_lines(src: &str) -> Vec<Line> {
        super::parse_lines(0, src).unwrap()
    }

    #[test]
    fn try_it_with_macros() {
        let int = parse_term(" 123");
        assert_eq!(int, term!(123));
        assert_eq!(int.offset(), 1);
        let s = parse_term(r#""string literal""#);
        assert_eq!(s, term!("string literal"));

        let t = parse_term(r#"true"#);
        assert_eq!(t, term!(true));

        let sym = parse_term(r#"foo_qwe"#);
        assert_eq!(sym, term!(sym!("foo_qwe")));

        let l = parse_term(r#"[foo, bar, baz]"#);
        assert_eq!(l, term!([sym!("foo"), sym!("bar"), sym!("baz")]));

        let exp = parse_term(r#"foo(a, b(c), "d")"#);
        assert_eq!(
            exp,
            term!(call!("foo", [sym!("a"), call!("b", [sym!("c")]), "d"]))
        );

        let exp2 = parse_term(r#"foo.a(b)"#);
        assert_eq!(
            exp2,
            term!(op!(Dot, term!(sym!("foo")), term!(call!("a", [sym!("b")])))),
            "{}",
            exp2.to_polar()
        );

        let exp3 = parse_term(r#"foo.bar(a, b(c.d(e,[f,g])))"#);
        assert_eq!(
            exp3,
            term!(op!(
                Dot,
                term!(sym!("foo")),
                term!(call!(
                    "bar",
                    [
                        sym!("a"),
                        call!(
                            "b",
                            [op!(
                                Dot,
                                term!(sym!("c")),
                                term!(call!("d", [sym!("e"), value!([sym!("f"), sym!("g")])]))
                            )]
                        )
                    ]
                ))
            )),
            "{}",
            exp3.to_polar()
        );
        let rule = parse_rule(r#"f(x) if g(x);"#);
        assert_eq!(rule, rule!("f", [sym!("x")] => call!("g", [sym!("x")])));
        let rule = parse_rule(r#"f(x);"#);
        assert_eq!(rule, rule!("f", [sym!("x")]));
    }

    #[test]
    fn parse_booleans() {
        assert_eq!(parse_query("true"), term!(true));
        assert_eq!(parse_query("false"), term!(false));
    }

    #[test]
    fn parse_integers() {
        assert_eq!(parse_query("123"), term!(123));
        assert_eq!(parse_query("0"), term!(0));
        assert_eq!(parse_query("+123"), term!(123));
        assert_eq!(parse_query("-123"), term!(-123));
    }

    #[test]
    fn parse_floats() {
        assert_eq!(parse_query("0.123"), term!(0.123));
        assert_eq!(parse_query("1.234"), term!(1.234));
        assert_eq!(parse_query("+1.234"), term!(1.234));
        assert_eq!(parse_query("-1.234"), term!(-1.234));
        assert_eq!(parse_query("-1.234e-56"), term!(-1.234e-56));
        assert_eq!(parse_query("-1.234e56"), term!(-1.234e56));
        assert_eq!(parse_query("inf"), term!(f64::INFINITY));
        assert_eq!(parse_query("-inf"), term!(f64::NEG_INFINITY));
        assert!(
            matches!(parse_query("nan").value(), Value::Number(crate::numerics::Numeric::Float(f)) if f.is_nan())
        );
    }

    #[test]
    fn test_parse_specializers() {
        let rule = parse_rule(r#"f(x: 1);"#);
        assert_eq!(rule, rule!("f", ["x"; 1]));

        let rule = parse_rule(r#"f(x: 1, y: [x]) if y = 2;"#);
        assert_eq!(
            rule,
            rule!("f", ["x" ; 1 , "y" ; value!([sym!("x")])] => op!(Unify, term!(sym!("y")), term!(2)))
        );

        // parenthesized => parse as a symbol
        let rule = parse_rule(r#"f(x: (y));"#);
        assert_eq!(rule, rule!("f", ["x"; value!(sym!("y"))]));

        // not parenthesized => parse as a type
        let rule = parse_rule(r#"f(x: y);"#);
        assert_eq!(rule, rule!("f", ["x"; value!(instance!("y"))]));
    }

    #[test]
    fn test_parse_file() {
        let f = r#"
        a(1);b(2);c(3);
        "#;
        let results = parse_rules(0, f).unwrap();
        assert_eq!(results[0].to_polar(), r#"a(1);"#);
        assert_eq!(results[1].to_polar(), r#"b(2);"#);
        assert_eq!(results[2].to_polar(), r#"c(3);"#);
    }

    #[test]
    fn test_parse_line() {
        let kb = r#"f(x) if x = 1;"#;
        let line = parse_lines(&kb);
        assert_eq!(
            line[0],
            Line::Rule(rule!("f", [sym!("x")] => op!(Unify, term!(sym!("x")), term!(1))))
        );
        let f = r#"?= f(1);"#;
        let line = parse_lines(&f);

        assert_eq!(line[0], Line::Query(term!(call!("f", [1]))));
    }

    #[test]
    fn test_parse_new() {
        let f = r#"a(x) if x = new Foo(a: 1);"#;
        let results = parse_rules(0, f).unwrap();
        assert_eq!(results[0].to_polar(), r#"a(x) if x = new Foo(a: 1);"#);
    }

    #[test]
    fn test_parse_new_boa_constructor() {
        let f = r#"a(x) if x = new Foo(1, 2);"#;
        let results = parse_rules(0, f).unwrap();
        assert_eq!(results[0].to_polar(), r#"a(x) if x = new Foo(1, 2);"#);

        // test trailing comma
        let f = r#"a(x) if x = new Foo(1,);"#;
        parse_rules(0, f).expect_err("parse error");
    }

    #[test]
    fn test_parse_new_mixed_args() {
        let f = r#"a(x) if x = new Foo(1, 2, bar: 3, baz:4);"#;
        let results = parse_rules(0, f).unwrap();
        assert_eq!(
            results[0].to_polar(),
            r#"a(x) if x = new Foo(1, 2, bar: 3, baz: 4);"#
        );
        let f = r#"a(x) if x = new Foo(bar: 3, baz: 4);"#;
        let results = parse_rules(0, f).unwrap();
        assert_eq!(
            results[0].to_polar(),
            r#"a(x) if x = new Foo(bar: 3, baz: 4);"#
        );

        let f = r#"a(x) if x = new Foo(bar: 3, baz: 4, 1, 2);"#;
        parse_rules(0, f).expect_err("parse error");

        // Don't allow kwargs in calls or dot ops.
        let f = r#"a(x) if f(x: 1)"#;
        parse_rules(0, f).expect_err("parse error");
        let f = r#"a(x) if x.f(x: 1)"#;
        parse_rules(0, f).expect_err("parse error");
    }

    #[test]
    fn test_parse_matches() {
        let term = parse_query("{} matches {}");
        assert_eq!(term.to_polar(), r#"{} matches {}"#);
        let _term = parse_query("{x: 1} matches {}");
    }

    #[test]
    fn test_parse_rest_vars() {
        let q = "[1, 2, *x] = [*rest]";
        assert_eq!(parse_query(q).to_polar(), q);

        assert!(matches!(
            super::parse_query(0, "[1, 2, 3] = [*rest, 3]").expect_err("parse error"),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::UnrecognizedToken { .. }),
                ..
            }
        ));

        assert!(matches!(
            super::parse_query(0, "[1, 2, *3] = [*rest]").expect_err("parse error"),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::UnrecognizedToken { .. }),
                ..
            }
        ));

        assert!(matches!(
            super::parse_query(0, "[1, *x, *y] = [*rest]").expect_err("parse error"),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::UnrecognizedToken { .. }),
                ..
            }
        ));

        let q = "[1, 2, 3] matches [1, 2, 3]";
        assert_eq!(parse_query(q).to_polar(), q, "{} -- {}", q, parse_query(q));

        let q = "[1, 2, 3] matches [1, *rest]";
        assert_eq!(parse_query(q).to_polar(), q, "{} -- {}", q, parse_query(q));
    }

    #[test]
    fn test_primitive_methods() {
        let q = r#""abc".startswith("a")"#;
        assert_eq!(
            parse_query(q),
            term!(op!(Dot, term!("abc"), term!(call!("startswith", ["a"])))),
        );

        let q = r#"x.("invalid-key")"#;
        assert_eq!(
            parse_query(q),
            term!(op!(Dot, term!(sym!("x")), term!("invalid-key"))),
        );
    }

    #[test]
    fn test_catching_wrong_types() {
        for bad_query in &[
            "f(x=1)",
            "x in [1, 2] < 2",
            "{x: 1 < 2}",
            "{x: 1 < 2}",
            "not 1",
            "1 and 2",
            "1 + print(\"x\")",
            "forall([1, 2, 3], x < 1)",
            "x = (1 or 2)",
            "x = (1 = 2)",
            "foo.bar(x or y)",
            "foo.bar(z: x or y)",
            "x = y = z",
            "x = y = 1",
            "x = 1 = z",
            "1 = y = z",
            "x = (1 and 2)",
            "(1 or 2) = x",
            "x = (not x)",
            "y matches z = x",
        ] {
            assert!(matches!(
                super::parse_query(0, bad_query).expect_err("parse error"),
                error::PolarError {
                    kind: error::ErrorKind::Parse(error::ParseError::WrongValueType { .. }),
                    ..
                }
            ));
        }
    }

    #[test]
    fn test_catching_wrong_types_return_errors() {
        for bad_query in &[
            "f(x=1)",
            "x in [1, 2] < 2",
            "{x: 1 < 2}",
            "{x: 1 < 2}",
            "not 1",
            "1 and 2",
            "1 + print(\"x\")",
            "forall([1, 2, 3], x < 1)",
            "x = (1 or 2)",
            "x = (1 = 2)",
            "foo.bar(x or y)",
            "foo.bar(z: x or y)",
            "x = y = z",
            "x = y = 1",
            "x = 1 = z",
            "1 = y = z",
            "x = (1 and 2)",
            "(1 or 2) = x",
            "x = (not x)",
            "y matches z = x",
        ] {
            let query = super::parse_query_with_errors(0, bad_query).unwrap();
            println!("{}", query.0.to_polar());
            println!("{}", query.1);
            // assert!(matches!(query, Ok((_t, _e))));
        }
    }
}
