use std::iter::Peekable;
use std::str::Chars;

use super::types::*;

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    EOF,
    LP,
    RP,
    QuestionMark,
    SemiColon,
    Name(String),
}

#[derive(Debug)]
pub enum ParseError {
    InvalidTokenCharacter { c: char },
    InvalidToken { expected: Token, got: Token },
    //InvalidTokenName { expected: Token, got: Token },
}

fn get_next_token(src: &mut Peekable<Chars>) -> Result<Token, ParseError> {
    'restart: loop {
        loop {
            match src.peek() {
                Some(' ') | Some('\n') | Some('\r') | Some('\t') => {
                    src.next();
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
                    _ => Err(ParseError::InvalidTokenCharacter { c }),
                }
            }
        };
        return t;
    }
}

struct Lexer<'a> {
    src: Peekable<Chars<'a>>,
    pub token: Token,
}

impl<'a> Lexer<'a> {
    fn new(s: &'a str) -> Result<Self, ParseError> {
        let mut src = s.chars().peekable();
        let token = get_next_token(&mut src)?;
        Ok(Lexer { src, token })
    }

    fn next_token(&mut self) -> Result<(), ParseError> {
        let token = get_next_token(&mut self.src)?;
        self.token = token;
        Ok(())
    }

    fn is_token(&mut self, t: Token) -> bool {
        t == self.token
    }

    fn match_token(&mut self, t: Token) -> Result<bool, ParseError> {
        if self.is_token(t) {
            self.next_token()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn expect_token(&mut self, t: Token) -> Result<(), ParseError> {
        if !self.match_token(t.clone())? {
            Err(ParseError::InvalidToken {
                expected: t,
                got: self.token.clone(),
            })
        } else {
            Ok(())
        }
    }

    fn expect_a_name(&mut self) -> Result<String, ParseError> {
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

    // fn is_name(&mut self, string: &str) -> bool {
    //     if let Token::Name(ref s) = self.token {
    //         s == string
    //     } else {
    //         false
    //     }
    // }

    // fn match_name(&mut self, string: &str) -> Result<bool, ParseError> {
    //     if self.is_name(string) {
    //         self.next_token()?;
    //         Ok(true)
    //     } else {
    //         Ok(false)
    //     }
    // }

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

pub fn parse_str(src: String) -> Result<Vec<Term>, ParseError> {
    // let mut lex = Lexer::new(&src)?;
    // let clauses = parse_polar_file(&mut lex)?;
    // Ok(clauses)
    Err(ParseError::InvalidTokenCharacter { c: '\0' })
}

#[cfg(test)]
mod tests {
    // use super::*;
    // #[test]
    // fn it_works() {
    //     let query = parse_str(
    //         r#"
    //         foo(a)
    //         "#
    //             .to_owned(),
    //     )
    //         .unwrap();
    //     let rule = parse_str(
    //         r#"
    //         foo(a);
    //         "#
    //             .to_owned(),
    //     )
    //         .unwrap();
    //     println!("{:?}", kb);
    //     println!("{:?}", query);
    // }
}
