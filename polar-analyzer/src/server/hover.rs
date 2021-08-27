use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent};
use tracing::{debug, error};

use crate::inspect::TermInfo;

use super::Backend;

impl Backend {
    pub async fn get_hover(&self, params: HoverParams) -> Option<Hover> {
        let polar = self.analyzer.read().await;
        let filename = self
            .uri_to_string(&params.text_document_position_params.text_document.uri)
            .await;
        let position = params.text_document_position_params.position;
        let offset = match polar.source_map.position_to_offset(&filename, position) {
            Some(offset) => offset,
            None => {
                error!("Asking for hover but the file {} doesn't exist", filename);
                return None;
            }
        };

        polar.get_symbol_at(&filename, offset).map(
            |TermInfo {
                 location,
                 details,
                 r#type,
                 name,
                 ..
             }| {
                debug!("Found symbol: {}", name);
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
}
