use lalrpop_util::{lalrpop_mod, ParseError};

lalrpop_mod!(
    #[allow(clippy::all, dead_code, unused_imports, unused_mut)]
    polar
);

use super::lexer::{self, loc_to_pos, Lexer};
use super::types::{self, *};

#[derive(Clone, Debug, PartialEq)]
pub enum Line {
    Rule(Rule),
    Query(Term),
}

lazy_static::lazy_static! {
    static ref LINES_PARSER: polar::LinesParser = polar::LinesParser::new();
    static ref QUERY_PARSER: polar::ExpParser = polar::ExpParser::new();
    static ref RULES_PARSER: polar::RulesParser = polar::RulesParser::new();
    static ref TERM_PARSER: polar::TermParser = polar::TermParser::new();
}

fn to_parse_error(
    src: &str,
    e: ParseError<usize, lexer::Token, types::ParseError>,
) -> types::ParseError {
    match e {
        ParseError::InvalidToken { location } => types::ParseError::InvalidToken {
            pos: loc_to_pos(src, location),
        },
        ParseError::UnrecognizedEOF { location, .. } => types::ParseError::UnrecognizedEOF {
            pos: loc_to_pos(src, location),
        },
        ParseError::UnrecognizedToken {
            token: (s, t, _), ..
        } => types::ParseError::UnrecognizedToken {
            token: t.to_string(),
            pos: loc_to_pos(src, s),
        },
        ParseError::ExtraToken { token: (s, t, _) } => types::ParseError::ExtraToken {
            token: t.to_string(),
            pos: loc_to_pos(src, s),
        },
        ParseError::User { error } => error,
    }
}

pub fn parse_term(src: &str) -> PolarResult<Term> {
    TERM_PARSER
        .parse(Lexer::new(src))
        .map_err(|e| to_parse_error(src, e).into())
}

pub fn parse_lines(src: &str) -> PolarResult<Vec<Line>> {
    LINES_PARSER
        .parse(Lexer::new(src))
        .map_err(|e| to_parse_error(src, e).into())
}

pub fn parse_query(src: &str) -> PolarResult<Term> {
    QUERY_PARSER
        .parse(Lexer::new(src))
        .map_err(|e| to_parse_error(src, e).into())
}

#[cfg(test)]
pub fn parse_rules(src: &str) -> PolarResult<Vec<Rule>> {
    RULES_PARSER
        .parse(Lexer::new(src))
        .map_err(|e| to_parse_error(src, e).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToPolarString;
    use pretty_assertions::assert_eq;

    #[test]
    fn try_it() {
        let int = polar::IntegerParser::new()
            .parse(Lexer::new(" 123"))
            .unwrap();
        assert_eq!(int.to_polar(), "123");
        assert_eq!(int.offset, 1);
        let s = polar::PolarStringParser::new()
            .parse(Lexer::new(r#""string literal""#))
            .unwrap();
        assert_eq!(s.to_polar(), r#""string literal""#);
        let t = polar::BooleanParser::new()
            .parse(Lexer::new(r#"true"#))
            .unwrap();
        assert_eq!(t.to_polar(), r#"true"#);
        let sym = polar::SymbolParser::new()
            .parse(Lexer::new(r#"foo_qwe"#))
            .unwrap();
        assert_eq!(sym.to_polar(), r#"foo_qwe"#);
        let l = polar::ExpParser::new()
            .parse(Lexer::new(r#"[foo, bar, baz]"#))
            .unwrap();
        assert_eq!(l.to_polar(), r#"[foo,bar,baz]"#);
        let exp = polar::ExpParser::new()
            .parse(Lexer::new(r#"foo(a, b(c), "d")"#))
            .unwrap();
        assert_eq!(exp.to_polar(), r#"foo(a,b(c),"d")"#);
        let exp2 = polar::ExpParser::new()
            .parse(Lexer::new(r#"foo.bar(a, b(c.d(e,[f,g])))"#))
            .unwrap();
        assert_eq!(exp2.to_polar(), r#"foo.bar(a,b(c.d(e,[f,g])))"#);
        let rule = polar::RuleParser::new()
            .parse(Lexer::new(r#"f(x) := g(x);"#))
            .unwrap();
        assert_eq!(rule.to_polar(), r#"f(x) := g(x);"#);
        let rule = polar::RuleParser::new()
            .parse(Lexer::new(r#"f(x);"#))
            .unwrap();
        assert_eq!(rule.to_polar(), r#"f(x);"#);
        let _instance = polar::InstanceLiteralParser::new()
            .parse(Lexer::new(r#"Foo{bar: 1, baz: y, biz: "hi"}"#))
            .unwrap();
        // This won't work. There's no ordering to fields. Need to use sam macros.
        // println!("{}", instance.to_polar());
        // assert_eq!(instance.to_polar(), r#"Foo{baz: y, biz: "hi", bar: 1}"#);
        let exp = polar::ExpParser::new()
            .parse(Lexer::new(r#"!foo"#))
            .unwrap();
        assert_eq!(exp.to_polar(), r#"!foo"#);
        let exp = polar::ExpParser::new()
            .parse(Lexer::new(r#"!foo"#))
            .unwrap();
        assert_eq!(exp.to_polar(), r#"!foo"#);
        let exp = polar::ExpParser::new()
            .parse(Lexer::new(r#"!a,b|c=d==(e+f)/g.h(i)"#))
            .unwrap();
        assert_eq!(exp.to_polar(), r#"!a,b|c=d==(e+f)/g.h(i)"#);
    }

    #[test]
    fn try_it_with_macros() {
        let int = polar::TermParser::new().parse(Lexer::new(" 123")).unwrap();
        assert_eq!(int, term!(123));
        assert_eq!(int.offset, 1);
        let s = polar::TermParser::new()
            .parse(Lexer::new(r#""string literal""#))
            .unwrap();
        assert_eq!(s, term!("string literal"));

        let t = polar::TermParser::new()
            .parse(Lexer::new(r#"true"#))
            .unwrap();
        assert_eq!(t, term!(true));

        let sym = polar::TermParser::new()
            .parse(Lexer::new(r#"foo_qwe"#))
            .unwrap();
        assert_eq!(sym, term!(sym!("foo_qwe")));

        let l = polar::TermParser::new()
            .parse(Lexer::new(r#"[foo, bar, baz]"#))
            .unwrap();
        assert_eq!(l, term!([sym!("foo"), sym!("bar"), sym!("baz")]));

        let exp = polar::ExpParser::new()
            .parse(Lexer::new(r#"foo(a, b(c), "d")"#))
            .unwrap();
        assert_eq!(
            exp,
            term!(pred!("foo", [sym!("a"), pred!("b", [sym!("c")]), "d"]))
        );

        let exp2 = polar::ExpParser::new()
            .parse(Lexer::new(r#"foo.a(b)"#))
            .unwrap();
        assert_eq!(
            exp2,
            term!(op!(Dot, term!(sym!("foo")), term!(pred!("a", [sym!("b")])))),
            "{}",
            exp2.to_polar()
        );

        let exp3 = polar::ExpParser::new()
            .parse(Lexer::new(r#"foo.bar(a, b(c.d(e,[f,g])))"#))
            .unwrap();
        assert_eq!(
            exp3,
            term!(op!(
                Dot,
                term!(sym!("foo")),
                term!(pred!(
                    "bar",
                    [
                        sym!("a"),
                        pred!(
                            "b",
                            [op!(
                                Dot,
                                term!(sym!("c")),
                                term!(pred!("d", [sym!("e"), value!([sym!("f"), sym!("g")])]))
                            )]
                        )
                    ]
                ))
            )),
            "{}",
            exp3.to_polar()
        );
        let rule = polar::RuleParser::new()
            .parse(Lexer::new(r#"f(x) := g(x);"#))
            .unwrap();
        assert_eq!(rule, rule!("f", [sym!("x")] => pred!("g", [sym!("x")])));
        let rule = polar::RuleParser::new()
            .parse(Lexer::new(r#"f(x);"#))
            .unwrap();
        assert_eq!(rule, rule!("f", [sym!("x")]));
    }

    #[test]
    fn parse_booleans() {
        assert_eq!(parse_query("true").unwrap(), term!(true));
        assert_eq!(parse_query("false").unwrap(), term!(false));
    }

    #[test]
    fn test_parse_specializers() {
        let rule = polar::RuleParser::new()
            .parse(Lexer::new(r#"f(x: 1);"#))
            .unwrap();
        assert_eq!(rule, rule!("f", ["x"; 1]));

        let rule = polar::RuleParser::new()
            .parse(Lexer::new(r#"f(x: 1, y: [x]) := y = 2;"#))
            .unwrap();
        assert_eq!(
            rule,
            rule!("f", ["x" ; 1 , "y" ; value!([sym!("x")])] => op!(Unify, term!(sym!("y")), term!(2)))
        );

        // parenthesized => parse as a symbol
        let rule = polar::RuleParser::new()
            .parse(Lexer::new(r#"f(x: (y));"#))
            .unwrap();
        assert_eq!(rule, rule!("f", ["x"; value!(sym!("y"))]));

        // not parenthesized => parse as a type
        let rule = polar::RuleParser::new()
            .parse(Lexer::new(r#"f(x: y);"#))
            .unwrap();
        assert_eq!(rule, rule!("f", ["x"; value!(instance!("y"))]));
    }

    #[test]
    fn test_parse_file() {
        let f = r#"
        a(1);b(2);c(3);
        "#;
        let results = parse_rules(f).unwrap();
        assert_eq!(results[0].to_polar(), r#"a(1);"#);
        assert_eq!(results[1].to_polar(), r#"b(2);"#);
        assert_eq!(results[2].to_polar(), r#"c(3);"#);
    }

    #[test]
    fn test_parse_line() {
        let kb = r#"f(x) := x = 1;"#;
        let line = parse_lines(&kb).unwrap();
        assert_eq!(
            line[0],
            Line::Rule(rule!("f", [sym!("x")] => op!(Unify, term!(sym!("x")), term!(1))))
        );
        let f = r#"?= f(1);"#;
        let line = parse_lines(&f).unwrap();

        assert_eq!(line[0], Line::Query(term!(pred!("f", [1]))));
    }
}
