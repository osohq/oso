use std::sync::Arc;

use lalrpop_util::{lalrpop_mod, ParseError};

use super::{
    error::{self, PolarResult},
    lexer::{self, Lexer, Token},
    resource_block::Production,
    rules::*,
    sources::Source,
    terms::*,
};

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

#[derive(Clone, Debug, PartialEq)]
pub enum Line {
    Rule(Rule),
    RuleType(Rule),
    Query(Term),
    ResourceBlock {
        keyword: Option<Term>,
        resource: Term,
        productions: Vec<Production>,
    },
}

fn lalrpop_error_to_polar_error(
    e: ParseError<usize, lexer::Token, error::ParseErrorKind>,
    source: Arc<Source>,
) -> error::PolarError {
    let kind = match e {
        ParseError::InvalidToken { location: loc } => error::ParseErrorKind::InvalidToken { loc },
        ParseError::UnrecognizedEOF { location: loc, .. } => {
            error::ParseErrorKind::UnrecognizedEOF { loc }
        }
        ParseError::UnrecognizedToken {
            token: (loc, t, _), ..
        } => match t {
            Token::Debug | Token::Cut | Token::In | Token::New => {
                error::ParseErrorKind::ReservedWord {
                    token: t.to_string(),
                    loc,
                }
            }
            _ => error::ParseErrorKind::UnrecognizedToken {
                token: t.to_string(),
                loc,
            },
        },
        ParseError::ExtraToken { token: (loc, t, _) } => error::ParseErrorKind::ExtraToken {
            token: t.to_string(),
            loc,
        },
        ParseError::User { error } => error,
    };
    error::ParseError { source, kind }.into()
}

pub fn parse_lines(source: Source) -> PolarResult<Vec<Line>> {
    let source = Arc::new(source);
    polar::LinesParser::new()
        .parse(&source, Lexer::new(&source.src))
        .map_err(|e| lalrpop_error_to_polar_error(e, source))
}

pub fn parse_query(query: &str) -> PolarResult<Term> {
    let source = Arc::new(Source::new(query));
    polar::TermParser::new()
        .parse(&source, Lexer::new(query))
        .map_err(|e| lalrpop_error_to_polar_error(e, source))
}

#[cfg(test)]
pub fn parse_rules(rules: &str) -> PolarResult<Vec<Rule>> {
    let source = Arc::new(Source::new(rules));
    polar::RulesParser::new()
        .parse(&source, Lexer::new(rules))
        .map_err(|e| lalrpop_error_to_polar_error(e, source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ParseErrorKind::*;
    use pretty_assertions::assert_eq;

    #[track_caller]
    fn parse_term(src: &str) -> Term {
        super::parse_query(src).unwrap()
    }

    #[track_caller]
    fn parse_term_error(src: &str) -> error::ParseErrorKind {
        super::parse_query(src).unwrap_err().unwrap_parse()
    }

    #[track_caller]
    fn parse_rule(src: &str) -> Rule {
        super::parse_rules(src).unwrap().pop().unwrap()
    }

    #[track_caller]
    fn parse_lines(src: &str) -> Vec<Line> {
        super::parse_lines(Source::new(src)).unwrap()
    }

    #[test]
    fn test_dot_lookups() {
        let exp = parse_term("a.b");
        assert_eq!(exp, term!(op!(Dot, term!(sym!("a")), term!("b"))));
    }

    #[test]
    fn try_it_with_macros() {
        let int = parse_term(" 123");
        assert_eq!(int, term!(123));
        assert_eq!(int.parsed_context().map(|context| context.left), Some(1));
        let s = parse_term(r#""string literal""#);
        assert_eq!(s, term!("string literal"));

        let t = parse_term("true");
        assert_eq!(t, term!(true));

        let sym = parse_term("foo_qwe");
        assert_eq!(sym, term!(sym!("foo_qwe")));

        let l = parse_term("[foo, bar, baz]");
        assert_eq!(l, term!([sym!("foo"), sym!("bar"), sym!("baz")]));

        super::parse_rules(r#"bar(a, c) if foo(a, b(c), "d")"#).expect_err("parse error");

        let exp2 = parse_term("foo.a(b)");
        assert_eq!(
            exp2,
            term!(op!(Dot, term!(sym!("foo")), term!(call!("a", [sym!("b")])))),
            "{}",
            exp2
        );
        let rule = parse_rule("f(x) if g(x);");
        assert_eq!(rule, rule!("f", [sym!("x")] => call!("g", [sym!("x")])));
        let rule = parse_rule("f(x);");
        assert_eq!(rule, rule!("f", [sym!("x")]));
    }

    #[test]
    fn parse_booleans() {
        assert_eq!(parse_term("true"), term!(true));
        assert_eq!(parse_term("false"), term!(false));
    }

    #[test]
    fn parse_integers() {
        assert_eq!(parse_term("123"), term!(123));
        assert_eq!(parse_term("0"), term!(0));
        assert_eq!(parse_term("+123"), term!(123));
        assert_eq!(parse_term("-123"), term!(-123));
    }

    #[test]
    fn parse_floats() {
        assert_eq!(parse_term("0.123"), term!(0.123));
        assert_eq!(parse_term("1.234"), term!(1.234));
        assert_eq!(parse_term("+1.234"), term!(1.234));
        assert_eq!(parse_term("-1.234"), term!(-1.234));
        assert_eq!(parse_term("-1.234e-56"), term!(-1.234e-56));
        assert_eq!(parse_term("-1.234e56"), term!(-1.234e56));
        assert_eq!(parse_term("inf"), term!(f64::INFINITY));
        assert_eq!(parse_term("-inf"), term!(f64::NEG_INFINITY));
        assert!(
            matches!(parse_term("nan").value(), Value::Number(crate::numerics::Numeric::Float(f)) if f.is_nan())
        );
    }

    #[test]
    fn test_parse_specializers() {
        let rule = parse_rule("f(x: 1);");
        assert_eq!(rule, rule!("f", ["x"; 1]));

        let rule = parse_rule("f(x: 1, y: [x]) if y = 2;");
        assert_eq!(
            rule,
            rule!("f", ["x" ; 1 , "y" ; value!([sym!("x")])] => op!(Unify, term!(sym!("y")), term!(2)))
        );

        // parse specializer as a type
        let rule = parse_rule("f(x: y);");
        assert_eq!(rule, rule!("f", ["x"; value!(instance!("y"))]));
    }

    #[test]
    fn test_parse_file() {
        let f = "a(1);b(2);c(3);";
        let results = super::parse_rules(f).unwrap();
        assert_eq!(results[0].to_string(), "a(1);");
        assert_eq!(results[1].to_string(), "b(2);");
        assert_eq!(results[2].to_string(), "c(3);");
    }

    #[test]
    fn test_parse_line() {
        let kb = "f(x) if x = 1;";
        let line = parse_lines(kb);
        assert_eq!(
            line[0],
            Line::Rule(rule!("f", [sym!("x")] => op!(Unify, term!(sym!("x")), term!(1))))
        );
        let f = "?= f(1);";
        let line = parse_lines(f);

        assert_eq!(line[0], Line::Query(term!(call!("f", [1]))));

        let rule_type = "type f(x: String);";
        let line = parse_lines(rule_type);
        assert_eq!(
            line[0],
            Line::RuleType(rule!("f", ["x"; value!(instance!("String"))]))
        );
    }

    #[test]
    fn test_rule_type_error() {
        let rule_type = r#"type f(x: String) if x = "bad";"#;
        super::parse_lines(Source::new(rule_type)).unwrap_err();
    }

    #[test]
    fn test_parse_new() {
        let f = "a(x) if x = new Foo(a: 1);";
        let results = super::parse_rules(f).unwrap();
        assert_eq!(results[0].to_string(), "a(x) if x = new Foo(a: 1);");
    }

    #[test]
    fn test_parse_new_boa_constructor() {
        let f = "a(x) if x = new Foo(1, 2);";
        let results = super::parse_rules(f).unwrap();
        assert_eq!(results[0].to_string(), "a(x) if x = new Foo(1, 2);");

        // test trailing comma
        let f = "a(x) if x = new Foo(1,);";
        super::parse_rules(f).expect_err("parse error");
    }

    #[test]
    fn test_parse_new_mixed_args() {
        let f = "a(x) if x = new Foo(1, 2, bar: 3, baz:4);";
        let results = super::parse_rules(f).unwrap();
        assert_eq!(
            results[0].to_string(),
            "a(x) if x = new Foo(1, 2, bar: 3, baz: 4);"
        );
        let f = "a(x) if x = new Foo(bar: 3, baz: 4);";
        let results = super::parse_rules(f).unwrap();
        assert_eq!(
            results[0].to_string(),
            "a(x) if x = new Foo(bar: 3, baz: 4);"
        );

        let f = "a(x) if x = new Foo(bar: 3, baz: 4, 1, 2);";
        super::parse_rules(f).expect_err("parse error");

        // Don't allow kwargs in calls or dot ops.
        let f = "a(x) if f(x: 1)";
        super::parse_rules(f).expect_err("parse error");
        let f = "a(x) if x.f(x: 1)";
        super::parse_rules(f).expect_err("parse error");
    }

    #[test]
    fn test_parse_matches() {
        let term = parse_term("{} matches {}");
        assert_eq!(term.to_string(), "{} matches {}");
        let term = parse_term("{x: 1} matches {}");
        assert_eq!(term.to_string(), "{x: 1} matches {}");
    }

    #[test]
    fn test_parse_rest_vars() {
        let q = "[1, 2, *x] = [*rest]";
        assert_eq!(parse_term(q).to_string(), q);

        let e = parse_term_error("[1, 2, 3] = [*rest, 3]");
        assert!(matches!(e, UnrecognizedToken { .. }));

        let e = parse_term_error("[1, 2, *3] = [*rest]");
        assert!(matches!(e, UnrecognizedToken { .. }));

        let e = parse_term_error("[1, *x, *y] = [*rest]");
        assert!(matches!(e, UnrecognizedToken { .. }));

        let q = "[1, 2, 3] matches [1, 2, 3]";
        assert_eq!(parse_term(q).to_string(), q, "{} -- {}", q, parse_term(q));

        let q = "[1, 2, 3] matches [1, *rest]";
        assert_eq!(parse_term(q).to_string(), q, "{} -- {}", q, parse_term(q));
    }

    #[test]
    fn test_primitive_methods() {
        let q = r#""abc".startswith("a")"#;
        assert_eq!(
            parse_term(q),
            term!(op!(Dot, term!("abc"), term!(call!("startswith", ["a"])))),
        );

        let q = r#"x.("invalid-key")"#;
        assert_eq!(
            parse_term(q),
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
            assert!(matches!(parse_term_error(bad_query), WrongValueType { .. }));
        }
    }

    #[test]
    fn trailing_commas() {
        let q = "{a: 1,}";
        let dict = term!(btreemap! { sym!("a") => term!(1)});
        assert_eq!(parse_term(q), dict);

        let q = "[1, 2,]";
        let list = term!([1, 2]);
        assert_eq!(parse_term(q), list);

        assert_eq!(
            parse_term("{a: 1,} = [1, 2,]"),
            term!(op!(Unify, dict, list))
        );
    }

    #[test]
    fn duplicate_keys() {
        let q = "{a: 1, a: 2}";
        assert!(matches!(parse_term_error(q), DuplicateKey { .. }));
    }
}
