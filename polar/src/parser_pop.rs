use lalrpop_util::lalrpop_mod;

lalrpop_mod!(polar);

use super::types::*;

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
        let l = polar::TermParser::new().parse(r#"(foo, bar, baz)"#).unwrap();
        assert_eq!(l.to_polar(), r#"(foo,bar,baz)"#);

    }
}