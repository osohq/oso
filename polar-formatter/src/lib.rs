pub mod ast;
pub mod parser;
pub mod pretty_print;

use pretty_print::{PrettyContext, ToDoc};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn format(file: String) -> Option<String> {
    let node = parser::parse_file(1, &file).ok()?;

    let mut context = PrettyContext::new(file.clone());

    let mut result = Vec::new();
    node.to_doc(&mut context).render(80, &mut result).unwrap();
    result.extend(context.print_trailing_comments().bytes());

    let result = String::from_utf8(result).unwrap();
    Some(
        result
            .split('\n')
            .map(|s| s.trim_end())
            .collect::<Vec<_>>()
            .join("\n"),
    )
}
