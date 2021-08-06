use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent};

use crate::{inspect::TermInfo, Polar};

pub fn get_hover(polar: &Polar, params: HoverParams) -> Option<Hover> {
    let filename = params
        .text_document_position_params
        .text_document
        .uri
        .to_string();
    let position = params.text_document_position_params.position;
    let offset = polar
        .source_map
        .position_to_offset(&filename, position)
        .expect("file not found");

    polar.get_symbol_at(&filename, offset).map(
        |TermInfo {
             location,
             details,
             r#type,
             name,
             ..
         }| {
            eprintln!("Found symbol: {}", name);
            let (_filename, start, end) = location;
            let range = polar.source_map.location_to_range(&filename, start, end);

            Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: details.unwrap_or_else(|| r#type.clone()),
                }),
                range,
            }
        },
    )
}
