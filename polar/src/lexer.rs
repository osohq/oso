use super::types::Symbol;
use std::str::{CharIndices, FromStr};

pub struct Lexer<'input> {
    c: Option<(usize, char)>,
    chars: CharIndices<'input>,
    buf: String,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        let mut chars = input.char_indices();
        let c = chars.next();
        let buf = String::new();
        Lexer { c, chars, buf }
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Clone, Debug)]
pub enum Token {
    Integer(i64),
    String(String),
    Boolean(bool),
    Symbol(Symbol),
    Colon,     // :
    Comma,     // ,
    LB,        // [
    RB,        // ]
    LP,        // (
    RP,        // )
    LCB,       // {
    RCB,       // }
    Dot,       // .
    Make,      // make
    Not,       // !
    Mul,       // *
    Div,       // /
    Add,       // +
    Sub,       // -
    Eq,        // ==
    Neq,       // !=
    Leq,       // <=
    Geq,       // >=
    Lt,        // <
    Gt,        // >
    Unify,     // =
    Pipe,      // |
    SemiColon, // ;
    Define,    // :=
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, String>; // @TODO: Error, not String

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.c {
                Some((_, ' ')) | Some((_, '\n')) | Some((_, '\r')) | Some((_, '\t')) => {
                    self.c = self.chars.next();
                }
                Some((_, '#')) => {
                    self.c = self.chars.next();
                    loop {
                        // @TODO: Handle windows line endings too.
                        if let Some((_, '\n')) = self.c {
                            self.c = self.chars.next();
                            break;
                        } else {
                            self.c = self.chars.next();
                        }
                    }
                }
                _ => break,
            };
        }

        // Parse tokens.
        match self.c {
            None => return None,
            Some((i, char)) => match char {
                // @TODO: Support all utf8 stuff like the last parser did.
                'a'..='z' | 'A'..='Z' | '_' => {
                    let start = i;
                    let mut last = i;
                    self.buf.clear();
                    self.buf.push(char);
                    self.c = self.chars.next();
                    loop {
                        if let Some((i, char)) = self.c {
                            match char {
                                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                                    self.buf.push(char);
                                    last = i;
                                    self.c = self.chars.next();
                                }
                                _ => break,
                            }
                        } else {
                            break;
                        }
                    }
                    if &self.buf == "true" {
                        return Some(Ok((start, Token::Boolean(true), last + 1)));
                    } else if &self.buf == "false" {
                        return Some(Ok((start, Token::Boolean(false), last + 1)));
                    } else {
                        return Some(Ok((start, Token::Symbol(Symbol::new(&self.buf)), last + 1)));
                    }
                }
                '"' => {
                    let start = i;
                    let mut last = i;
                    self.buf.clear();
                    self.c = self.chars.next();
                    loop {
                        if let Some((i, char)) = self.c {
                            match char {
                                // @TODO: Escaped things.
                                '"' => {
                                    self.c = self.chars.next();
                                    last = i;
                                    break;
                                }
                                _ => {
                                    self.buf.push(char);
                                    self.c = self.chars.next();
                                }
                            }
                        } else {
                            todo!("Error, hit end of file before closing quote")
                        }
                    }
                    return Some(Ok((start, Token::String(self.buf.clone()), last + 1)));
                }
                '0'..='9' => {
                    let start = i;
                    let mut last = i;
                    self.buf.clear();
                    self.buf.push(char);
                    self.c = self.chars.next();
                    loop {
                        if let Some((i, char)) = self.c {
                            match char {
                                '0'..='9' => {
                                    self.buf.push(char);
                                    self.c = self.chars.next();
                                    last = i;
                                }
                                _ => break,
                            }
                        } else {
                            break;
                        }
                    }
                    if let Ok(int) = i64::from_str(&self.buf) {
                        return Some(Ok((start, Token::Integer(int), last + 1)));
                    } else {
                        todo!("Error")
                    }
                }
                ':' => {
                    let start = i;
                    self.c = self.chars.next();
                    if let Some((_, '=')) = self.c {
                        self.c = self.chars.next();
                        return Some(Ok((start, Token::Define, start + 2)));
                    } else {
                        return Some(Ok((start, Token::Colon, start + 1)));
                    }
                }
                '=' => {
                    let start = i;
                    self.c = self.chars.next();
                    if let Some((_, '=')) = self.c {
                        self.c = self.chars.next();
                        return Some(Ok((start, Token::Eq, start + 2)));
                    } else {
                        return Some(Ok((start, Token::Unify, start + 1)));
                    }
                }
                '<' => {
                    let start = i;
                    self.c = self.chars.next();
                    if let Some((_, '=')) = self.c {
                        self.c = self.chars.next();
                        return Some(Ok((start, Token::Leq, start + 2)));
                    } else {
                        return Some(Ok((start, Token::Lt, start + 1)));
                    }
                }
                '>' => {
                    let start = i;
                    self.c = self.chars.next();
                    if let Some((_, '=')) = self.c {
                        self.c = self.chars.next();
                        return Some(Ok((start, Token::Geq, start + 2)));
                    } else {
                        return Some(Ok((start, Token::Gt, start + 1)));
                    }
                }
                '!' => {
                    let start = i;
                    self.c = self.chars.next();
                    if let Some((_, '=')) = self.c {
                        self.c = self.chars.next();
                        return Some(Ok((start, Token::Neq, start + 2)));
                    } else {
                        return Some(Ok((start, Token::Not, start + 1)));
                    }
                }
                '|' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::Pipe, i + 1)));
                }
                ',' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::Comma, i + 1)));
                }
                '[' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::LB, i + 1)));
                }
                ']' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::RB, i + 1)));
                }
                '{' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::LCB, i + 1)));
                }
                '}' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::RCB, i + 1)));
                }
                '(' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::LP, i + 1)));
                }
                ')' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::RP, i + 1)));
                }
                '.' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::Dot, i + 1)));
                }
                '+' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::Add, i + 1)));
                }
                '-' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::Sub, i + 1)));
                }
                '*' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::Mul, i + 1)));
                }
                '/' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::Div, i + 1)));
                }
                ';' => {
                    self.c = self.chars.next();
                    return Some(Ok((i, Token::SemiColon, i + 1)));
                }
                _ => {
                    return Some(Err(format!(
                        "Lexer error: Invalid token character: '{}'",
                        char
                    )))
                }
            },
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_infinate_looper() {
        let f = " 123";
        let mut lexer = Lexer::new(f);
        assert!(matches!(
            lexer.next(),
            Some(Ok((_, Token::Integer(123), _)))
        ));
        assert!(matches!(lexer.next(), None));
    }

    #[test]
    fn test_lexer() {
        let f = r#"hello "world" 12345 < + <= { ] =99 #comment
            more;"#;
        let mut lexer = Lexer::new(&f);
        assert!(
            matches!(lexer.next(), Some(Ok((0, Token::Symbol(hello), 5))) if hello == Symbol::new("hello"))
        );
        assert!(
            matches!(lexer.next(), Some(Ok((6, Token::String(world), 13))) if &world == "world")
        );
        assert!(matches!(
            lexer.next(),
            Some(Ok((14, Token::Integer(12345), 19)))
        ));
        assert!(matches!(lexer.next(), Some(Ok((20, Token::Lt, 21)))));
        assert!(matches!(lexer.next(), Some(Ok((22, Token::Add, 23)))));
        assert!(matches!(lexer.next(), Some(Ok((24, Token::Leq, 26)))));
        assert!(matches!(lexer.next(), Some(Ok((27, Token::LCB, 28)))));
        assert!(matches!(lexer.next(), Some(Ok((29, Token::RB, 30)))));
        assert!(matches!(lexer.next(), Some(Ok((31, Token::Unify, 32)))));
        assert!(matches!(
            lexer.next(),
            Some(Ok((32, Token::Integer(99), 34)))
        ));
        assert!(
            matches!(lexer.next(), Some(Ok((56, Token::Symbol(more), 60))) if more == Symbol::new("more"))
        );
        assert!(matches!(lexer.next(), Some(Ok((60, Token::SemiColon, 61)))));
        assert!(matches!(lexer.next(), None));
    }
}
