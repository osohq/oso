use lalrpop_util::lalrpop_mod;

lalrpop_mod!(polar);

use super::types::*;

pub fn parse_query(src: &str) -> Predicate {
    // @TODO: Errors
    polar::PredicateParser::new().parse(src).unwrap()
}

pub fn parse_file(src: &str) -> Vec<Rule> {
    // @TODO: Errors
    polar::RulesParser::new().parse(src).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_it() {
        let int = polar::IntegerParser::new().parse(" 123").unwrap();
        assert_eq!(int.to_polar(), "123");
        assert_eq!(int.offset, 1);
        let s = polar::PolarStringParser::new().parse(r#""string literal""#).unwrap();
        assert_eq!(s.to_polar(), r#""string literal""#);
        let t = polar::BooleanParser::new().parse(r#"true"#).unwrap();
        assert_eq!(t.to_polar(), r#"true"#);
        let sym = polar::SymbolParser::new().parse(r#"foo_qwe"#).unwrap();
        assert_eq!(sym.to_polar(), r#"foo_qwe"#);
        let l = polar::ExpParser::new().parse(r#"(foo, bar, baz)"#).unwrap();
        assert_eq!(l.to_polar(), r#"(foo,bar,baz)"#);
        let exp = polar::ExpParser::new().parse(r#"foo(a, b(c), "d")"#).unwrap();
        assert_eq!(exp.to_polar(), r#"foo(a,b(c),"d")"#);
        let exp2 = polar::ExpParser::new().parse(r#"foo.bar(a, b(c.d(e,(f,g))))"#).unwrap();
        assert_eq!(exp2.to_polar(), r#".(foo,bar,a,b(.(c,d,e,(f,g))))"#);
        let rule = polar::RuleParser::new().parse(r#"f(x) := g(x);"#).unwrap();
        assert_eq!(rule.to_polar(), r#"f(x) := (g(x));"#);
        let rule = polar::RuleParser::new().parse(r#"f(x);"#).unwrap();
        assert_eq!(rule.to_polar(), r#"f(x) := ();"#);
    }

    #[test]
    fn test_parse_file() {
        let f = r#"
        a(1);b(2);c(3);
        "#;
        let results = parse_file(f);
        assert_eq!(results[0].to_polar(), r#"a(1) := ();"#);
        assert_eq!(results[1].to_polar(), r#"b(2) := ();"#);
        assert_eq!(results[2].to_polar(), r#"c(3) := ();"#);
    }
}