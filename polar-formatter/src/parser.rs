use crate::ast::Node;
use lalrpop_util::{lalrpop_mod, ParseError};
use polar_core::lexer::{self, Lexer, Token};

pub enum ValueOrLogical {
    Value(Term),
    Logical(Term),
    Either(Term),
}

lalrpop_mod!(
    #[allow(clippy::all, dead_code, unused_imports, unused_mut)]
    polar
);

use polar_core::error;
use polar_core::resource_block::Production;
use polar_core::rules::*;
use polar_core::terms::*;

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

// pub fn parse_lines(src_id: u64, src: &str) -> Result<Vec<Line>, error::ParseError> {
//     polar::LinesParser::new()
//         .parse(src_id, Lexer::new(src))
//         .map_err(to_parse_error)
// }

// pub fn parse_query(src_id: u64, src: &str) -> Result<Term, error::ParseError> {
//     polar::TermParser::new()
//         .parse(src_id, Lexer::new(src))
//         .map_err(to_parse_error)
// }

pub fn parse_file(src_id: u64, src: &str) -> Result<Node, error::ParseError> {
    polar::FileParser::new()
        .parse(src_id, Lexer::new(src))
        .map_err(to_parse_error)
}

pub fn parse_expression(src_id: u64, src: &str) -> Result<Node, error::ParseError> {
    polar::ExpressionParser::new()
        .parse(src_id, Lexer::new(src))
        .map_err(to_parse_error)
}
