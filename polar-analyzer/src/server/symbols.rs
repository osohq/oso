use lsp_types::{DocumentSymbolParams, DocumentSymbolResponse, Location, SymbolInformation};

use crate::inspect::RuleInfo;

use super::Backend;

impl Backend {
    pub async fn get_document_symbols(
        &self,
        params: DocumentSymbolParams,
    ) -> Option<DocumentSymbolResponse> {
        let filename = self.uri_to_string(&params.text_document.uri).await;
        tracing::trace!("polar read lock");
        let polar = self.analyzer.read().await;
        tracing::trace!("polar read lock acquired");
        let rules = polar.get_rule_info(&filename);
        let rules = rules
            .into_iter()
            .map(
                |RuleInfo {
                     symbol, location, ..
                 }| {
                    let range = polar
                        .source_map
                        .location_to_range(&filename, location.1, location.2)
                        .unwrap_or_default();

                    // `deprecated` is deprecated, but we're not using
                    // it so we'll allow using the deprecated `deprecated` field.
                    #[allow(deprecated)]
                    SymbolInformation {
                        name: symbol,
                        kind: lsp_types::SymbolKind::Method,
                        location: Location {
                            uri: params.text_document.uri.clone(),
                            range,
                        },
                        tags: None,
                        deprecated: None,
                        container_name: None,
                    }
                },
            )
            .collect();

        Some(DocumentSymbolResponse::Flat(rules))
    }
}
