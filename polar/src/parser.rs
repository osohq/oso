use std::iter::Peekable;
use std::str::Chars;
use std::collections::HashMap;

use super::types::*;


#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    EOF,
    LP,
    RP,
    Dot,
    SemiColon,
    Comma,
    Define,
    Name(String),
    String(String),
    Int(i64),
}

#[derive(Debug)]
pub enum ParseError {
    InvalidTokenCharacter(char),
    InvalidToken { expected: Token, got: Token },
    ErrorParsingNumber(String),
    //InvalidTokenName { expected: Token, got: Token },
    Unimplemented
}

fn get_next_token(src: &mut Peekable<Chars>) -> Result<Token, ParseError> {
    loop {
        match src.peek() {
            Some(' ') | Some('\n') | Some('\r') | Some('\t') => {
                src.next();
            },
            Some('#') => {
                src.next();
                loop {
                    match src.peek() {
                        // @TODO: Handle \r\n for windows.
                        None | Some('\n') => break,
                        _ => src.next()
                    };
                }
            }
            _ => break,
        }
    }

    let t = match src.peek().cloned() {
        None => Ok(Token::EOF),
        Some(c) => {
            match c {
                // Name
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut name = String::new();
                    name.push(c);
                    src.next();
                    loop {
                        if let Some(c) = src.peek().cloned() {
                            match c {
                                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                                    name.push(c);
                                    src.next();
                                    continue;
                                }
                                _ => (),
                            }
                        }
                        break;
                    }
                    Ok(Token::Name(name))
                }
                '0'..='9' => {
                    let mut num = String::new();
                    num.push(c);
                    src.next();
                    loop {
                        if let Some(c) = src.peek().cloned() {
                            match c {
                                '0'..='9' => {
                                    num.push(c);
                                    src.next();
                                    continue;
                                }
                                _ => (),
                            }
                        }
                        break;
                    }
                    if let Ok(i) = num.parse::<i64>() {
                        Ok(Token::Int(i))
                    } else {
                        Err(ParseError::ErrorParsingNumber(num))
                    }

                }
                '"' => {
                    let mut s = String::new();
                    src.next();
                    // @TODO: Handle escapes.
                    loop {
                        match src.peek().cloned() {
                            Some('"') | None => {
                                src.next();
                                break;
                            }
                            Some(c) => {
                                s.push(c);
                                src.next();
                            }
                        }
                    }
                    Ok(Token::String(s))
                }
                '(' => {
                    src.next();
                    Ok(Token::LP)
                }
                ')' => {
                    src.next();
                    Ok(Token::RP)
                }
                ';' => {
                    src.next();
                    Ok(Token::SemiColon)
                }
                ',' => {
                    src.next();
                    Ok(Token::Comma)
                }
                '.' => {
                    src.next();
                    Ok(Token::Dot)
                }
                ':' => {
                    src.next();
                    if let Some('=') = src.peek() {
                        src.next();
                        Ok(Token::Define)
                    } else {
                        Err(ParseError::InvalidTokenCharacter(':'))
                    }
                }
                _ => Err(ParseError::InvalidTokenCharacter(c)),
            }
        }
    };
    return t;
}

pub struct Lexer<'a> {
    src: Peekable<Chars<'a>>,
    pub token: Token,
}

impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Result<Self, ParseError> {
        let mut src = s.chars().peekable();
        let token = get_next_token(&mut src)?;
        Ok(Lexer { src, token })
    }

    pub fn next_token(&mut self) -> Result<(), ParseError> {
        let token = get_next_token(&mut self.src)?;
        self.token = token;
        Ok(())
    }

    pub fn is_token(&self, t: Token) -> bool {
        t == self.token
    }

    pub fn match_token(&mut self, t: Token) -> Result<bool, ParseError> {
        if self.is_token(t) {
            self.next_token()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn expect_token(&mut self, t: Token) -> Result<(), ParseError> {
        if !self.match_token(t.clone())? {
            Err(ParseError::InvalidToken {
                expected: t,
                got: self.token.clone(),
            })
        } else {
            Ok(())
        }
    }

    pub fn expect_a_name(&mut self) -> Result<String, ParseError> {
        if let Token::Name(n) = self.token.clone() {
            self.next_token()?;
            Ok(n)
        } else {
            Err(ParseError::InvalidToken {
                expected: Token::Name("".to_string()),
                got: self.token.clone(),
            })
        }
    }

    pub fn is_name(&self, string: &str) -> bool {
        if let Token::Name(ref s) = self.token {
            s == string
        } else {
            false
        }
    }

    pub fn match_name(&mut self, string: &str) -> Result<bool, ParseError> {
        if self.is_name(string) {
            self.next_token()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // fn expect_name(&mut self, string: &str) -> Result<(), ParseError> {
    //     if !self.match_name(string)? {
    //         Err(ParseError::InvalidTokenName {
    //             expected: Token::Name(string.to_string()),
    //             got: self.token.clone(),
    //         })
    //     } else {
    //         Ok(())
    //     }
    // }
}
//
// fn parse_atom(lexer: &mut Lexer) -> Result<Atom, ParseError> {
//     let variable = lexer.match_token(Token::QuestionMark)?;
//     let name = lexer.expect_a_name()?;
//     if variable {
//         Ok(Atom::Var(name))
//     } else {
//         Ok(Atom::Name(name))
//     }
// }
//
// fn parse_predicate(mut lexer: &mut Lexer) -> Result<Predicate, ParseError> {
//     let head = lexer.expect_a_name()?;
//     let mut args = vec![];
//     lexer.expect_token(Token::LP)?;
//     while !lexer.is_token(Token::RP) {
//         let arg = parse_atom(&mut lexer)?;
//         args.push(arg);
//     }
//     lexer.expect_token(Token::RP)?;
//     lexer.expect_token(Token::SemiColon)?; // Manditory for now.
//     Ok(Predicate { head, args })
// }
//
// fn parse_fact(mut lexer: &mut Lexer) -> Result<Clause, ParseError> {
//     let pred = parse_predicate(&mut lexer)?;
//     Ok(Clause::Fact { pred })
// }
//
// fn parse_clause(mut lexer: &mut Lexer) -> Result<Clause, ParseError> {
//     // @TODO: Rules
//     parse_fact(&mut lexer)
// }
//
// fn parse_polar_file(mut lexer: &mut Lexer) -> Result<Vec<Clause>, ParseError> {
//     let mut clauses = vec![];
//     while !lexer.is_token(Token::EOF) {
//         clauses.push(parse_clause(&mut lexer)?)
//     }
//     Ok(clauses)
// }

pub fn parse_term(mut lexer: &mut Lexer) -> Result<Term, ParseError> {
    if lexer.is_token(Token::LP) {
        let list = parse_term_list(&mut lexer)?;
        Ok(Term{id: 0, value: Value::List(list)})
    } else if lexer.match_name("true")? {
        lexer.next_token()?;
        Ok(Term{id: 0, value: Value::Boolean(true)})
    } else if lexer.match_name("false")? {
        lexer.next_token()?;
        Ok(Term{id: 0, value: Value::Boolean(true)})
    } else if let Token::Int(i) = lexer.token {
        lexer.next_token()?;
        Ok(Term{id: 0, value: Value::Integer(i)})
    } else {
        match lexer.token.clone() {
            Token::String(s) => {
                lexer.next_token()?;
                Ok(Term{id: 0, value: Value::String(s)})
            },
            Token::Name(n) => {
                lexer.next_token()?;if lexer.match_token(Token::Dot)? {
                    let mut args = vec![];
                    args.push(Term{id: 0, value: Value::Symbol(Symbol(n))});
                    let attribute = lexer.expect_a_name()?;
                    args.push(Term{id: 0, value: Value::Symbol(Symbol(attribute))});
                    if lexer.is_token(Token::LP) {
                        let call_args = parse_term_list(&mut lexer)?;
                        for a in call_args {
                            args.push(a);
                        }
                    }
                    let name = ".".to_owned();
                    Ok(Term{id: 0, value: Value::Call(Predicate{name, args})})
                } else {
                    if lexer.is_token(Token::LP) {
                        let call_args = parse_term_list(&mut lexer)?;
                        Ok(Term{id: 0, value: Value::Call(Predicate{name: n, args: call_args})})
                    } else {
                        Ok(Term{id: 0, value: Value::Symbol(Symbol(n))})
                    }
                }
            },
            // @TODO: Instance
            _ => Err(ParseError::Unimplemented)
        }
    }
}

pub fn parse_term_list(mut lexer: &mut Lexer) -> Result<TermList, ParseError> {
    lexer.expect_token(Token::LP)?;
    let mut terms = vec![];
    while !lexer.is_token(Token::RP) {
        let val = parse_term(&mut lexer)?;
        terms.push(val);
        lexer.match_token(Token::Comma)?;
    }
    lexer.expect_token(Token::RP)?;
    Ok(terms)
}

pub fn parse_predicate(mut lexer: &mut Lexer) -> Result<Predicate, ParseError> {
    let name = lexer.expect_a_name()?;
    let args = parse_term_list(&mut lexer)?;
    Ok(Predicate{name, args})
}

pub fn parse_rule(mut lexer: &mut Lexer) -> Result<Rule, ParseError> {
    let head = parse_predicate(&mut lexer)?;
    let mut body = vec![];
    if lexer.match_token(Token::Define)? {
        while !lexer.is_token(Token::SemiColon) {
            let term = parse_term(&mut lexer)?;
            body.push(term);
            lexer.match_token(Token::Comma)?;
        }
        lexer.expect_token(Token::SemiColon)?;
    }
    Ok(Rule{name: head.name, params: head.args, body})
}

pub fn parse_query(src: String) -> Result<Predicate, ParseError> {
    let mut lex = Lexer::new(&src)?;
    let pred = parse_predicate(&mut lex)?;
    Ok(pred)
}

pub fn parse_source(src: String) -> Result<Vec<Rule>, ParseError> {
    let mut lex = Lexer::new(&src)?;
    let mut rules = vec![];
    while !lex.is_token(Token::EOF) {
        let rule = parse_rule(&mut lex)?;
        rules.push(rule);
    }
    lex.expect_token(Token::EOF)?;
    Ok(rules)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::Config;
    use super::*;

    #[test]
    fn test_simple_lex() {
        let input: &'static str = r#"abc hello "string" (what, up) #comment
foo;"#;
        let mut l = Lexer::new(input).unwrap();

        let mut assert_token = |token| {
            assert_eq!(l.token, token);
            l.next_token().unwrap();
        };

        assert_token(Token::Name("abc".to_string()));
        assert_token(Token::Name("hello".to_string()));
        assert_token(Token::String("string".to_string()));
        assert_token(Token::LP);
        assert_token(Token::Name("what".to_string()));
        assert_token(Token::Comma);
        assert_token(Token::Name("up".to_string()));
        assert_token(Token::RP);
        assert_token(Token::Name("foo".to_string()));
        assert_token(Token::SemiColon);
        assert_token(Token::EOF);
    }

    #[test]
    fn test_example_predicates() {
        let input: &'static str = r#"foo(a, b, "c", d.e(f.g("x")))"#;
        let mut l = Lexer::new(input).unwrap();
        let pred = parse_predicate(&mut l);
        println!("{}", input);
        println!("{:#?}", pred);
    }

    #[test]
    fn test_example_file() {
        let input: &'static str = r#"
foo(a,b) := bar(baz(1,2,3,(4,5,6),"hello")), what(up);
bar(a,b) := bar(baz(1,2,3,(4,5,6),"hello")),
    what(up),
    tho;
        "#;
        let rules = parse_source(input.to_owned()).unwrap();
        println!("{}", input);
        for rule in rules {
            println!("{}", rule.to_polar());
        }
    }

    #[test]
    fn test_example_rule() {
        let input: &'static str = r#"
        foo(a,b) := bar(1);
        "#;
        let rules = parse_source(input.to_owned()).unwrap();
        println!("{}", input);
        for rule in rules {
            println!("{}", rule.to_polar());
            println!("{:#?}", rule);
        }
    }

    #[test]
    fn test_dot_operator() {
        let input: &'static str = r#"
        test(foo.bar(1,2,3), hello)
        "#;
        let pred = parse_query(input.to_owned()).unwrap();
        println!("{}", input);
        println!("{}", pred.to_polar());
    }

    // @TODO: Get proptest working.
    // fn print_value(val: &Value) -> String {
    //     match val {
    //         Value::Integer(i) => format!("{}", i),
    //         Value::String(s) => format!("\"{}\"", s),
    //         Value::Boolean(true) => "true".to_owned(),
    //         Value::Boolean(false) => "false".to_owned(),
    //         Value::Instance(i) => "instance".to_owned(), // @TODO
    //         Value::Call(predicate) => "call".to_owned(), // @TODO
    //         Value::List(terms) => "list".to_owned(), // @TODO
    //         Value::Symbol(symbol) => "symbol".to_owned(), // @TODO
    //     }
    // }
    // // @TODO: Needs to be a recursive BoxedStrategy
    //
    //
    // fn gen_value() -> BoxedStrategy<Value> {
    //     let leaf = prop_oneof![
    //         any::<i64>().prop_map(Value::Integer),
    //         //gen_name().prop_map(Value::Variable),
    //         //"[^\"]*".prop_map(Value::StringValue),
    //         any::<bool>().prop_map(Value::BoolValue),
    //     ];
    //     leaf.prop_recursive(
    //         3,
    //         50,
    //         10,
    //         |inner| prop_oneof![
    //             prop::collection::vec(innter.clone(), 0..10).prop_map(Value::List),
    //
    //         ]
    //     )
    // }
    //
    // prop_compose! {
    //     fn gen_name()(n in "")
    // }

}
