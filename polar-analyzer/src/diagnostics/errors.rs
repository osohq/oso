use crate::PolarError;

pub fn find_parse_errors(src: &str) -> Vec<PolarError> {
    let parse_result = polar_core::parser::parse_file_with_errors(0, src);

    match parse_result {
        Ok((_lines, errors)) => errors.into_iter().map(PolarError::from).collect(),
        Err(e) => vec![e.into()],
    }
}
