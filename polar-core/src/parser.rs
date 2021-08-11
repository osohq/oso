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
pub struct ResourceNamespace {
    pub name: Symbol,
    // TODO(gj): capture source info
    // TODO(gj): maybe HashSet instead of Vec so we can easily catch duplicates?
    pub roles: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
    pub implications: Option<Vec<(String, String)>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Line {
    Rule(Rule),
    Query(Term),
    ResourceNamespace(ResourceNamespace),
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
    polar::TermParser::new()
        .parse(0, Lexer::new(src))
        .map_err(|e| to_parse_error(e).into())
}

pub fn parse_lines(src_id: u64, src: &str) -> PolarResult<Vec<Line>> {
    polar::LinesParser::new()
        .parse(src_id, Lexer::new(src))
        .map_err(|e| to_parse_error(e).into())
}

pub fn parse_query(src_id: u64, src: &str) -> PolarResult<Term> {
    polar::TermParser::new()
        .parse(src_id, Lexer::new(src))
        .map_err(|e| to_parse_error(e).into())
}

#[cfg(test)]
pub fn parse_rules(src_id: u64, src: &str) -> PolarResult<Vec<Rule>> {
    polar::RulesParser::new()
        .parse(src_id, Lexer::new(src))
        .map_err(|e| to_parse_error(e).into())
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
    fn test_dot_lookups() {
        let exp = parse_term("a.b");
        assert_eq!(exp, term!(op!(Dot, term!(sym!("a")), term!("b"))));
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

        parse_rules(0, r#"bar(a, c) if foo(a, b(c), "d")"#).expect_err("parse error");

        let exp2 = parse_term(r#"foo.a(b)"#);
        assert_eq!(
            exp2,
            term!(op!(Dot, term!(sym!("foo")), term!(call!("a", [sym!("b")])))),
            "{}",
            exp2.to_polar()
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
        let line = parse_lines(kb);
        assert_eq!(
            line[0],
            Line::Rule(rule!("f", [sym!("x")] => op!(Unify, term!(sym!("x")), term!(1))))
        );
        let f = r#"?= f(1);"#;
        let line = parse_lines(f);

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
        assert_eq!(term.to_polar(), "{} matches {}");
        let term = parse_query("{x: 1} matches {}");
        assert_eq!(term.to_polar(), "{x: 1} matches {}");
    }

    #[test]
    fn test_parse_namespace_with_no_declarations() {
        assert!(matches!(
            super::parse_lines(0, "Org{}").unwrap_err(),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::IntegerOverflow {
                    token,
                    ..
                }),
                ..
            } if token == "Org: must declare roles and/or permissions"
        ));
    }

    #[test]
    fn test_parse_namespace_with_empty_declarations() {
        assert!(matches!(
            super::parse_lines(0, "Org{roles=[];}").unwrap_err(),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::IntegerOverflow {
                    token,
                    ..
                }),
                ..
            } if token == "must declare at least one role"
        ));
        assert!(matches!(
            super::parse_lines(0, "Org{permissions=[];}").unwrap_err(),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::IntegerOverflow {
                    token,
                    ..
                }),
                ..
            } if token == "must declare at least one permission"
        ));
    }

    #[test]
    fn test_parse_namespace_with_only_roles() {
        assert_eq!(
            parse_lines(r#"Org{roles=["owner",];}"#)[0],
            Line::ResourceNamespace(ResourceNamespace {
                name: sym!("Org"),
                roles: Some(vec!["owner".to_owned()]),
                permissions: None,
                implications: None,
            })
        );
        assert_eq!(
            parse_lines(r#"Org{roles=["owner","member",];}"#)[0],
            Line::ResourceNamespace(ResourceNamespace {
                name: sym!("Org"),
                roles: Some(vec!["owner".to_owned(), "member".to_owned()]),
                permissions: None,
                implications: None,
            })
        );
    }

    #[test]
    fn test_parse_namespace_with_only_implications() {
        assert!(matches!(
            super::parse_lines(0, r#"Org{"member" if "owner";}"#).unwrap_err(),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::IntegerOverflow {
                    token,
                    ..
                }),
                ..
            } if token == "Org: cannot declare implications without roles and/or permissions"
        ));
    }

    #[test]
    fn test_parse_namespace_with_implications_above_declarations() {
        assert!(matches!(
            super::parse_lines(0, r#"Org {
                     "member" if "owner";
                     roles=["owner","member"];
                }"#).unwrap_err(),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::IntegerOverflow {
                    token,
                    ..
                }),
                ..
            } if token == "Org: move declarations (roles and permissions) above implications"
        ));

        assert!(matches!(
            super::parse_lines(0, r#"Org {
                     "create_repo" if "invite";
                     permissions=["invite","create_repo"];
                }"#).unwrap_err(),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::IntegerOverflow {
                    token,
                    ..
                }),
                ..
            } if token == "Org: move declarations (roles and permissions) above implications"
        ));
    }

    #[test]
    fn test_parse_namespace_with_roles_and_role_implications() {
        assert_eq!(
            parse_lines(
                r#"Org {
                     roles=["owner","member"];
                     "member" if "owner";
                }"#
            )[0],
            Line::ResourceNamespace(ResourceNamespace {
                name: sym!("Org"),
                roles: Some(vec!["owner".to_owned(), "member".to_owned()]),
                permissions: None,
                implications: Some(vec![("member".to_owned(), "owner".to_owned())]),
            })
        );
    }

    #[test]
    fn test_parse_namespace_with_permissions_but_no_implications() {
        assert!(matches!(
            super::parse_lines(0, r#"Org{permissions=["invite","create_repo"];}"#).unwrap_err(),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::IntegerOverflow {
                    token,
                    ..
                }),
                ..
            } if token == r#"Org: declared "invite" permission must be involved in at least one implication"#
        ));
    }

    #[test]
    fn test_parse_namespace_with_permission_not_involved_in_implication() {
        assert!(matches!(
            super::parse_lines(0, r#"Org {
                permissions=["invite","create_repo","ban"];
                "invite" if "ban";
            }"#).unwrap_err(),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::IntegerOverflow {
                    token,
                    ..
                }),
                ..
            } if token == r#"Org: declared "create_repo" permission must be involved in at least one implication"#
        ));
    }

    #[test]
    fn test_parse_namespace_with_implied_term_not_declared_locally() {
        assert!(matches!(
            super::parse_lines(0, r#"Org {
                roles=["owner"];
                "member" if "owner";
            }"#).unwrap_err(),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::IntegerOverflow {
                    token,
                    ..
                }),
                ..
            } if token == r#"Org: implied term "member" must be declared as a permission or role"#
        ));
    }

    #[test]
    fn test_parse_namespace_with_implier_term_not_declared_locally() {
        assert!(matches!(
            super::parse_lines(0, r#"Org {
                roles=["member"];
                "member" if "owner";
            }"#).unwrap_err(),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::IntegerOverflow {
                    token,
                    ..
                }),
                ..
                    // TODO(gj): make this error more visual; something like
                    //           my_file.polar:37 "member" if "owner";
                    //                                        ^^^^^^^ "owner" must be declared as a permission or role
            } if token == r#"Org: implier term "owner" must be declared as a permission or role"#
        ));
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
    fn trailing_commas() {
        let q = r#"{a: 1,}"#;
        let dict = term!(btreemap! { sym!("a") => term!(1)});
        assert_eq!(parse_term(q), dict);

        let q = r#"[1, 2,]"#;
        let list = term!([1, 2]);
        assert_eq!(parse_term(q), list);

        assert_eq!(
            parse_query(r#"{a: 1,} = [1, 2,]"#),
            term!(op!(Unify, dict, list))
        );
    }

    #[test]
    fn duplicate_keys() {
        let q = r#"{a: 1, a: 2}"#;
        assert!(matches!(
            super::parse_query(0, q).expect_err("parse error"),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::DuplicateKey { .. }),
                ..
            }
        ));
    }
}
