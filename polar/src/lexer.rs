use super::error::{ErrorContext, ParseError};
use super::types::{Source, Symbol};
use std::iter::Peekable;
use std::str::{CharIndices, FromStr};

pub type SrcPos = (usize, usize);

// Take a location in a string and return the row and column.
pub fn loc_to_pos(src: &str, loc: usize) -> SrcPos {
    let mut row = 0;
    let mut col = 0;
    let mut chars = src.chars();
    for _ in 0..loc {
        let c = chars.next();
        match c {
            Some('\n') => {
                row += 1;
                col = 0;
            }
            Some(_) => col += 1,
            None => panic!("loc is longer than the string."),
        }
    }
    (row, col)
}

pub fn make_context(source: &Source, loc: usize) -> Option<ErrorContext> {
    let (row, column) = loc_to_pos(&source.src, loc);
    Some(ErrorContext {
        source: source.clone(),
        row,
        column,
    })
}

pub struct Lexer<'input> {
    c: Option<(usize, char)>,
    chars: Peekable<CharIndices<'input>>,
    buf: String,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        let mut chars = input.char_indices().peekable();
        let c = chars.next();
        let buf = String::new();
        Lexer { c, chars, buf }
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Clone, Debug)]
pub enum Token {
    Integer(i64),
    Float(f64),
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
    New,       // new
    Bang,      // !
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
    Query,     // ?=
    In,        // in
    Cut,       // cut
    Debug,     // debug()
    Print,     // print()
    Isa,       // isa
    ForAll,    // forall
    If,        // if
    And,       // and
    Or,        // or
    Not,       // not
    Matches,   // matches
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Token::Integer(i) => i.to_string(),
            Token::Float(f) => f.to_string(),
            Token::String(s) => s.clone(),
            Token::Boolean(b) => b.to_string(),
            Token::Symbol(sym) => sym.0.clone(),
            Token::Colon => ":".to_owned(),         // :
            Token::Comma => ",".to_owned(),         // ,
            Token::LB => "[".to_owned(),            // [
            Token::RB => "]".to_owned(),            // ]
            Token::LP => "(".to_owned(),            // (
            Token::RP => ")".to_owned(),            // )
            Token::LCB => "{".to_owned(),           // {
            Token::RCB => "}".to_owned(),           // }
            Token::Dot => ".".to_owned(),           // .
            Token::New => "new".to_owned(),         // new
            Token::Bang => "!".to_owned(),          // !
            Token::Mul => "*".to_owned(),           // *
            Token::Div => "/".to_owned(),           // /
            Token::Add => "+".to_owned(),           // +
            Token::Sub => "-".to_owned(),           // -
            Token::Eq => "==".to_owned(),           // ==
            Token::Neq => "!=".to_owned(),          // !=
            Token::Leq => "<=".to_owned(),          // <=
            Token::Geq => ">=".to_owned(),          // >=
            Token::Lt => "<".to_owned(),            // <
            Token::Gt => ">".to_owned(),            // >
            Token::Unify => "-".to_owned(),         // =
            Token::Pipe => "|".to_owned(),          // |
            Token::SemiColon => ";".to_owned(),     // ;
            Token::Define => ":=".to_owned(),       // :=
            Token::Query => "?=".to_owned(),        // ?=
            Token::In => "in".to_owned(),           // in
            Token::Cut => "cut".to_owned(),         // cut
            Token::Debug => "debug".to_owned(),     // debug
            Token::Print => "print".to_owned(),     // print
            Token::Isa => "isa".to_owned(),         // isa
            Token::ForAll => "forall".to_owned(),   // forall
            Token::If => "if".to_owned(),           // if
            Token::And => "and".to_owned(),         // and
            Token::Or => "or".to_owned(),           // or
            Token::Not => "not".to_owned(),         // not
            Token::Matches => "matches".to_owned(), // matches
        }
    }
}

impl<'input> Lexer<'input> {
    #[inline]
    fn skip_whitespace(&mut self) {
        loop {
            match self.c {
                Some((_, ' ')) | Some((_, '\n')) | Some((_, '\r')) | Some((_, '\t')) => {
                    self.c = self.chars.next();
                }
                Some((_, '#')) => {
                    self.c = self.chars.next();
                    loop {
                        match self.c {
                            None | Some((_, '\r')) | Some((_, '\n')) => {
                                break;
                            }
                            _ => {
                                self.c = self.chars.next();
                            }
                        }
                    }
                }
                _ => break,
            };
        }
    }

    #[inline]
    fn scan_symbol(&mut self, i: usize, chr: char) -> Option<Spanned<Token, usize, ParseError>> {
        let start = i;
        let mut last = i;
        self.buf.clear();
        self.buf.push(chr);
        self.c = self.chars.next();

        while let Some((i, char)) = self.c {
            match char {
                x if x == '_' || (!x.is_ascii_punctuation() && !x.is_ascii_whitespace()) => {
                    self.buf.push(char);
                    last = i;
                    self.c = self.chars.next();
                }
                ':' => {
                    if let Some((i, ':')) = self.chars.peek() {
                        self.buf.push_str("::");
                        last = *i;
                        self.chars.next();
                        self.c = self.chars.next();
                    } else {
                        break;
                    }
                }
                '?' => {
                    self.buf.push(char);
                    last = i;
                    self.c = self.chars.next();
                    break; // ? is only valid as the last char in a symbol.
                }
                _ => break,
            }
        }
        if let Some((i, _)) = &self.buf.char_indices().rev().nth(1) {
            if &self.buf[*i..] == "::" {
                return Some(Err(ParseError::InvalidTokenCharacter {
                    token: self.buf.clone(),
                    c: ':',
                    loc: last,
                    context: None,
                }));
            }
        }

        if &self.buf == "true" {
            Some(Ok((start, Token::Boolean(true), last + 1)))
        } else if &self.buf == "false" {
            Some(Ok((start, Token::Boolean(false), last + 1)))
        } else if &self.buf == "new" {
            Some(Ok((start, Token::New, last + 1)))
        } else if &self.buf == "in" {
            Some(Ok((start, Token::In, last + 1)))
        } else if &self.buf == "cut" {
            Some(Ok((start, Token::Cut, last + 1)))
        } else if &self.buf == "debug" {
            Some(Ok((start, Token::Debug, last + 1)))
        } else if &self.buf == "print" {
            Some(Ok((start, Token::Print, last + 1)))
        } else if &self.buf == "isa" {
            Some(Ok((start, Token::Isa, last + 1)))
        } else if &self.buf == "forall" {
            Some(Ok((start, Token::ForAll, last + 1)))
        } else if &self.buf == "if" {
            Some(Ok((start, Token::If, last + 1)))
        } else if &self.buf == "and" {
            Some(Ok((start, Token::And, last + 1)))
        } else if &self.buf == "or" {
            Some(Ok((start, Token::Or, last + 1)))
        } else if &self.buf == "not" {
            Some(Ok((start, Token::Not, last + 1)))
        } else if &self.buf == "matches" {
            Some(Ok((start, Token::Matches, last + 1)))
        } else {
            Some(Ok((start, Token::Symbol(Symbol::new(&self.buf)), last + 1)))
        }
    }

    #[inline]
    fn scan_string(&mut self, i: usize) -> Option<Spanned<Token, usize, ParseError>> {
        let start = i;
        let last;
        self.buf.clear();
        self.c = self.chars.next();
        loop {
            if let Some((i, char)) = self.c {
                match char {
                    '\n' => {
                        return Some(Err(ParseError::InvalidTokenCharacter {
                            token: self.buf.clone(),
                            c: char,
                            loc: i,
                            context: None,
                        }))
                    }
                    '"' => {
                        self.c = self.chars.next();
                        last = i;
                        break;
                    }
                    '\\' => {
                        self.c = self.chars.next();
                        if let Some((_, char)) = self.c {
                            let escaped_char = match char {
                                'n' => '\n',
                                'r' => '\r',
                                't' => '\t',
                                '0' => '\0',
                                c => c,
                            };
                            self.buf.push(escaped_char);
                        } else {
                            return Some(Err(ParseError::InvalidTokenCharacter {
                                token: self.buf.clone(),
                                c: '\0',
                                loc: i,
                                context: None,
                            }));
                        }
                        self.c = self.chars.next();
                    }
                    _ => {
                        self.buf.push(char);
                        self.c = self.chars.next();
                    }
                }
            } else {
                return Some(Err(ParseError::InvalidTokenCharacter {
                    token: self.buf.clone(),
                    c: '\0',
                    loc: i,
                    context: None,
                }));
            }
        }
        Some(Ok((start, Token::String(self.buf.clone()), last + 1)))
    }

    #[inline]
    fn push_char(&mut self, c: char) {
        self.buf.push(c);
        self.c = self.chars.next();
    }

    #[inline]
    fn match_digits(&mut self, mut last: usize) -> usize {
        while let Some((i, char)) = self.c {
            match char {
                '0'..='9' => {
                    self.push_char(char);
                    last = i;
                }
                _ => break,
            }
        }
        last
    }

    #[inline]
    fn scan_number(&mut self, i: usize, chr: char) -> Option<Spanned<Token, usize, ParseError>> {
        let start = i;
        let mut last = i;
        self.buf.clear();
        self.buf.push(chr);
        self.c = self.chars.next();
        let mut parse_as_float = false;

        last = self.match_digits(last);

        if let Some((i, '.')) = self.c {
            self.push_char('.');
            last = i;
            parse_as_float = true;

            last = self.match_digits(last);
        }

        if let Some((i, char)) = self.c {
            match char {
                'e' | 'E' => {
                    self.push_char(char);
                    last = i;
                    parse_as_float = true;

                    last = self.match_digits(last);

                    if let Some((i, char)) = self.c {
                        match char {
                            '+' | '-' => {
                                self.push_char(char);
                                last = i;

                                last = self.match_digits(last);
                            }
                            _ => (),
                        }
                    }
                }
                _ => (),
            }
        }

        if parse_as_float {
            if let Ok(f) = f64::from_str(&self.buf) {
                Some(Ok((start, Token::Float(f), last + 1)))
            } else {
                Some(Err(ParseError::InvalidFloat {
                    token: self.buf.clone(),
                    loc: start,
                    context: None,
                }))
            }
        } else if let Ok(int) = i64::from_str(&self.buf) {
            Some(Ok((start, Token::Integer(int), last + 1)))
        } else {
            Some(Err(ParseError::IntegerOverflow {
                token: self.buf.clone(),
                loc: start,
                context: None,
            }))
        }
    }

    /// Scan a one character operator to token.
    #[inline]
    fn scan_1c_op(&mut self, i: usize, token: Token) -> Option<Spanned<Token, usize, ParseError>> {
        self.c = self.chars.next();
        Some(Ok((i, token, i + 1)))
    }

    /// Scan a two character operator to token.
    #[inline]
    fn scan_2c_op(
        &mut self,
        i: usize,
        next_char: char,
        token: Token,
    ) -> Option<Spanned<Token, usize, ParseError>> {
        let start = i;
        self.c = self.chars.next();
        match self.c {
            Some((_, chr)) if chr == next_char => {
                self.c = self.chars.next();
                Some(Ok((start, token, start + 2)))
            }
            Some((i, chr)) => Some(Err(ParseError::InvalidTokenCharacter {
                token: token.to_string(),
                c: chr,
                loc: i,
                context: None,
            })),
            _ => Some(Err(ParseError::InvalidTokenCharacter {
                token: token.to_string(),
                c: '\0',
                loc: start + 1,
                context: None,
            })),
        }
    }

    /// Scan an operator to token unless next_char is the next char in which case scan to next_token.
    #[inline]
    fn scan_1c_or_2c_op(
        &mut self,
        i: usize,
        token: Token,
        next_char: char,
        next_token: Token,
    ) -> Option<Spanned<Token, usize, ParseError>> {
        let start = i;
        self.c = self.chars.next();
        match self.c {
            Some((_, chr)) if chr == next_char => {
                self.c = self.chars.next();
                Some(Ok((start, next_token, start + 2)))
            }
            _ => Some(Ok((start, token, start + 1))),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, ParseError>; // @TODO: Error, not String

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        match self.c {
            None => None,
            Some((i, char)) => match char {
                x if x == '_' || (!x.is_ascii_punctuation() && !x.is_ascii_digit()) => {
                    self.scan_symbol(i, char)
                }
                '"' => self.scan_string(i),
                '0'..='9' => self.scan_number(i, char),
                ':' => self.scan_1c_or_2c_op(i, Token::Colon, '=', Token::Define),
                '=' => self.scan_1c_or_2c_op(i, Token::Unify, '=', Token::Eq),
                '<' => self.scan_1c_or_2c_op(i, Token::Lt, '=', Token::Leq),
                '>' => self.scan_1c_or_2c_op(i, Token::Gt, '=', Token::Geq),
                '!' => self.scan_1c_or_2c_op(i, Token::Bang, '=', Token::Neq),
                '?' => self.scan_2c_op(i, '=', Token::Query),
                '|' => self.scan_1c_op(i, Token::Pipe),
                ',' => self.scan_1c_op(i, Token::Comma),
                '[' => self.scan_1c_op(i, Token::LB),
                ']' => self.scan_1c_op(i, Token::RB),
                '{' => self.scan_1c_op(i, Token::LCB),
                '}' => self.scan_1c_op(i, Token::RCB),
                '(' => self.scan_1c_op(i, Token::LP),
                ')' => self.scan_1c_op(i, Token::RP),
                '.' => self.scan_1c_op(i, Token::Dot),
                '+' => self.scan_1c_op(i, Token::Add),
                '-' => self.scan_1c_op(i, Token::Sub),
                '*' => self.scan_1c_op(i, Token::Mul),
                '/' => self.scan_1c_op(i, Token::Div),
                ';' => self.scan_1c_op(i, Token::SemiColon),
                _ => Some(Err(ParseError::InvalidTokenCharacter {
                    token: "".to_owned(),
                    c: char,
                    loc: i,
                    context: None,
                })),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_loc_to_pos() {
        let src = "hello\nworld\r\nsomething";
        assert_eq!(loc_to_pos(src, 4), (0, 4));
        assert_eq!(loc_to_pos(src, 6), (1, 0));
        assert_eq!(loc_to_pos(src, 13), (2, 0));
        assert_eq!(loc_to_pos(src, 18), (2, 5));
    }

    #[test]
    fn lex_infinite_loop_bugs() {
        let f = " 123";
        let mut lexer = Lexer::new(f);
        assert!(matches!(
            lexer.next(),
            Some(Ok((_, Token::Integer(123), _)))
        ));
        assert!(matches!(lexer.next(), None));
        let s = "123 #comment";
        let mut lexer = Lexer::new(s);
        assert!(matches!(
            lexer.next(),
            Some(Ok((_, Token::Integer(123), _)))
        ));
        assert!(matches!(lexer.next(), None));
    }

    #[test]
    fn test_line_endings() {
        let f = "foo\nbar\rbaz\r\n#comment\n#windowscomment\r\n123";
        let mut lexer = Lexer::new(f);
        assert!(matches!(lexer.next(), Some(Ok((_, Token::Symbol(_), _)))));
        assert!(matches!(lexer.next(), Some(Ok((_, Token::Symbol(_), _)))));
        assert!(matches!(lexer.next(), Some(Ok((_, Token::Symbol(_), _)))));
        assert!(matches!(
            lexer.next(),
            Some(Ok((_, Token::Integer(123), _)))
        ));
    }

    #[test]
    fn test_escapes() {
        let s = r#"
            "this is a \"sub\" string"
        "#;
        let mut lexer = Lexer::new(&s);
        let tok = lexer.next();
        assert!(
            matches!(tok, Some(Ok((_, Token::String(s), _))) if &s == r#"this is a "sub" string"#)
        );
    }

    #[test]
    fn test_emoji() {
        let s = r#"
            "💯" 💯
        "#;
        let mut lexer = Lexer::new(&s);
        assert!(
            matches!(lexer.next(), Some(Ok((13, Token::String(hunnid), 19))) if hunnid == "💯")
        );
        assert!(
            matches!(lexer.next(), Some(Ok((20, Token::Symbol(hunnid), 21))) if hunnid == Symbol::new("💯"))
        );
    }

    #[test]
    fn test_symbol_with_trailing_question_mark() {
        let s = "foo?";
        let mut lexer = Lexer::new(&s);
        assert!(
            matches!(lexer.next(), Some(Ok((0, Token::Symbol(question), 4))) if question == Symbol::new("foo?"))
        );

        let s = "foo??";
        let mut lexer = Lexer::new(&s);
        lexer.next();
        assert!(matches!(
            lexer.next(),
            Some(Err(ParseError::InvalidTokenCharacter {
                token: t,
                c: '\u{0}',
                loc: 5,
                context: None
            })) if &t == "?="
        ));
    }

    #[test]
    fn test_symbol_colons() {
        let s = "foo:bar";
        let mut lexer = Lexer::new(&s);
        assert!(
            matches!(lexer.next(), Some(Ok((0, Token::Symbol(x), 3))) if x == Symbol::new("foo"))
        );
        assert!(matches!(lexer.next(), Some(Ok((3, Token::Colon, 4)))));
        assert!(
            matches!(lexer.next(), Some(Ok((4, Token::Symbol(x), 7))) if x == Symbol::new("bar"))
        );
        assert!(matches!(lexer.next(), None));

        let s = "foo::bar";
        let mut lexer = Lexer::new(&s);
        assert!(
            matches!(lexer.next(), Some(Ok((0, Token::Symbol(x), 8))) if x == Symbol::new("foo::bar"))
        );
        assert!(matches!(lexer.next(), None));

        let s = "foo:::bar";
        let mut lexer = Lexer::new(&s);
        assert!(matches!(
            lexer.next(),
            Some(Err(ParseError::InvalidTokenCharacter {
                token: x,
                c: ':',
                loc: 4,
                context: None
            })) if &x == "foo::"
        ));
    }

    #[test]
    fn test_symbol_question_marks() {
        let s = "foo??";
        let mut lexer = Lexer::new(&s);
        assert!(
            matches!(lexer.next(), Some(Ok((0, Token::Symbol(x), 4))) if x == Symbol::new("foo?"))
        );
        assert!(matches!(
            lexer.next(),
            Some(Err(ParseError::InvalidTokenCharacter {
                token: x,
                c: '\u{0}',
                loc: 5,
                context: None
            })) if &x == "?="
        ));
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn test_lexer() {
        let f = r#"hello "world" 12345 < + <= { ] =99 #comment
            more; in; Ruby::Namespace"#;
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
        assert!(matches!(lexer.next(), Some(Ok((62, Token::In, 64)))));
        assert!(matches!(lexer.next(), Some(Ok((64, Token::SemiColon, 65)))));
        assert!(
            matches!(lexer.next(), Some(Ok((66, Token::Symbol(ruby_namespace), 81))) if ruby_namespace == Symbol::new("Ruby::Namespace"))
        );
        assert!(matches!(lexer.next(), None));
    }

    #[test]
    fn test_numbers() {
        let f = "1+2";
        let mut lexer = Lexer::new(&f);
        assert!(matches!(lexer.next(), Some(Ok((0, Token::Integer(1), 1)))));
        assert!(matches!(lexer.next(), Some(Ok((1, Token::Add, 2)))));
        assert!(matches!(lexer.next(), Some(Ok((2, Token::Integer(2), 3)))));

        let f = "0123";
        let mut lexer = Lexer::new(&f);
        assert!(matches!(
            lexer.next(),
            Some(Ok((0, Token::Integer(123), 4)))
        ));
    }
}
