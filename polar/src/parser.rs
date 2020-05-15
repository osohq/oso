use lalrpop_util::lalrpop_mod;

lalrpop_mod!(polar);

use super::types::*;

pub fn parse_query(src: &str) -> PolarResult<Term> {
    // @TODO: Better Errors
    polar::ExpParser::new()
        .parse(src)
        .map_err(|e| PolarError::Parse(e.to_string()))
}

pub fn parse_rules(src: &str) -> PolarResult<Vec<Rule>> {
    // @TODO: Better Errors
    polar::RulesParser::new()
        .parse(src)
        .map_err(|e| PolarError::Parse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_it() {
        let int = polar::IntegerParser::new().parse(" 123").unwrap();
        assert_eq!(int.to_polar(), "123");
        assert_eq!(int.offset, 1);
        let s = polar::PolarStringParser::new()
            .parse(r#""string literal""#)
            .unwrap();
        assert_eq!(s.to_polar(), r#""string literal""#);
        let t = polar::BooleanParser::new().parse(r#"true"#).unwrap();
        assert_eq!(t.to_polar(), r#"true"#);
        let sym = polar::SymbolParser::new().parse(r#"foo_qwe"#).unwrap();
        assert_eq!(sym.to_polar(), r#"foo_qwe"#);
        let l = polar::ExpParser::new().parse(r#"[foo, bar, baz]"#).unwrap();
        assert_eq!(l.to_polar(), r#"[foo,bar,baz]"#);
        let exp = polar::ExpParser::new()
            .parse(r#"foo(a, b(c), "d")"#)
            .unwrap();
        assert_eq!(exp.to_polar(), r#"foo(a,b(c),"d")"#);
        let exp2 = polar::ExpParser::new()
            .parse(r#"foo.bar(a, b(c.d(e,[f,g])))"#)
            .unwrap();
        assert_eq!(exp2.to_polar(), r#"foo.bar(a,b(c.d(e,[f,g])))"#);
        let rule = polar::RuleParser::new().parse(r#"f(x) := g(x);"#).unwrap();
        assert_eq!(rule.to_polar(), r#"f(x) := g(x);"#);
        let rule = polar::RuleParser::new().parse(r#"f(x);"#).unwrap();
        assert_eq!(rule.to_polar(), r#"f(x);"#);
        let instance = polar::InstanceLiteralParser::new()
            .parse(r#"Foo{bar: 1, baz: y, biz: "hi"}"#)
            .unwrap();
        // This won't work. There's no ordering to fields. Need to use sam macros.
        // println!("{}", instance.to_polar());
        // assert_eq!(instance.to_polar(), r#"Foo{baz: y, biz: "hi", bar: 1}"#);
        let exp = polar::ExpParser::new().parse(r#"!foo"#).unwrap();
        assert_eq!(exp.to_polar(), r#"!foo"#);
        let exp = polar::ExpParser::new().parse(r#"!foo"#).unwrap();
        assert_eq!(exp.to_polar(), r#"!foo"#);
        let exp = polar::ExpParser::new()
            .parse(r#"!a,b|c=d==(e+f)/g.h(i)"#)
            .unwrap();
        assert_eq!(exp.to_polar(), r#"!a,b|c=d==(e+f)/g.h(i)"#);
    }

    // //#[test]
    // fn try_it_with_macros() {
    //     let int = polar::TermParser::new().parse(" 123").unwrap();
    //     assert_eq!(int, term!(value!(123)));
    //     assert_eq!(int.offset, 1);
    //     let s = polar::TermParser::new()
    //         .parse(r#""string literal""#)
    //         .unwrap();
    //     assert_eq!(s, term!(value!("string literal")));

    //     let t = polar::TermParser::new().parse(r#"true"#).unwrap();
    //     assert_eq!(t, term!(value!(true)));

    //     let sym = polar::TermParser::new().parse(r#"foo_qwe"#).unwrap();
    //     assert_eq!(sym, term!(value!(@sym "foo_qwe")));

    //     let l = polar::TermParser::new()
    //         .parse(r#"[foo, bar, baz]"#)
    //         .unwrap();
    //     assert_eq!(l, term!(value!(@tl sym!("foo"), sym!("bar"), sym!("baz"))));

    //     let exp = polar::ExpParser::new()
    //         .parse(r#"foo(a, b(c), "d")"#)
    //         .unwrap();
    //     assert_eq!(
    //         exp,
    //         term!(value!(pred!("foo", sym!("a"), pred!("b", sym!("c")), "d")))
    //     );

    //     /* let exp2 = polar::ExpParser::new()
    //         .parse(r#"foo.bar(a, b(c.d(e,[f,g])))"#)
    //         .unwrap();
    //     assert_eq!(
    //         exp2,
    //         term!(value!(pred!(
    //             ".",
    //             sym!("foo"),
    //             sym!("bar"), // <- TODO: is this right? the attribute is looked up as a string?
    //             sym!("a"),
    //             pred!(
    //                 "b",
    //                 pred!(
    //                     ".",
    //                     sym!("c"),
    //                     sym!("d"), // <- TODO: is this right? the attribute is looked up as a string?
    //                     sym!("e"),
    //                     value!(@tl sym!("f"), sym!("g"))
    //                 )
    //             )
    //         )))
    //     ); */
    //     let rule = polar::RuleParser::new().parse(r#"f(x) := g(x);"#).unwrap();
    //     assert_eq!(rule, rule!("f", sym!("x") => pred!("g", sym!("x"))));
    //     let rule = polar::RuleParser::new().parse(r#"f(x);"#).unwrap();
    //     assert_eq!(rule, rule!("f", sym!("x") => ));
    // }

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
}
