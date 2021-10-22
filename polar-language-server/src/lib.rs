use std::collections::HashMap;

use lsp_types::{
    notification::{
        DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument,
        DidSaveTextDocument, Initialized, Notification,
    },
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidOpenTextDocumentParams, FileChangeType, FileEvent, Position, PublishDiagnosticsParams,
    Range, TextDocumentItem, Url, VersionedTextDocumentIdentifier,
};
use polar_core::{error::PolarError, polar::Polar, sources::Source};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct PolarLanguageServer {
    documents: HashMap<Url, TextDocumentItem>,
    polar: Polar,
    send_diagnostics_callback: js_sys::Function,
}

fn range_from_polar_error(e: &PolarError) -> Range {
    let line = e.context.as_ref().map_or(0, |c| c.row) as u32;
    let character = e.context.as_ref().map_or(0, |c| c.column) as u32;
    Range {
        start: Position { line, character },
        end: Position { line, character },
    }
}

/// Public API exposed via WASM.
#[wasm_bindgen]
impl PolarLanguageServer {
    #[wasm_bindgen(constructor)]
    pub fn new(send_diagnostics_callback: &js_sys::Function) -> Self {
        console_error_panic_hook::set_once();

        Self {
            documents: HashMap::new(),
            polar: Polar::default(),
            send_diagnostics_callback: send_diagnostics_callback.clone(),
        }
    }

    /// Catch-all handler for notifications sent by the LSP client.
    ///
    /// This function receives a notification's `method` and `params` and dispatches to the
    /// appropriate handler function based on `method`.
    #[wasm_bindgen(js_class = PolarLanguageServer, js_name = onNotification)]
    pub fn on_notification(&mut self, method: &str, params: JsValue) {
        match method {
            DidOpenTextDocument::METHOD => {
                self.on_did_open_text_document(serde_wasm_bindgen::from_value(params).unwrap())
            }
            DidChangeTextDocument::METHOD => {
                self.on_did_change_text_document(serde_wasm_bindgen::from_value(params).unwrap())
            }
            DidChangeWatchedFiles::METHOD => {
                self.on_did_change_watched_files(serde_wasm_bindgen::from_value(params).unwrap())
            }
            // We don't care when a document is saved -- we already have the updated state thanks
            // to `DidChangeTextDocument`.
            DidSaveTextDocument::METHOD => (),
            // We don't care when a document is closed -- we care about all Polar files in a
            // workspace folder regardless of which ones remain open.
            DidCloseTextDocument::METHOD => (),
            // Nothing to do when we receive the `Initialized` notification.
            Initialized::METHOD => (),
            _ => {
                log(&format!(
                    "[WASM] on_notification\n\t{} => {:?}",
                    method, params
                ));
            }
        }
    }
}

fn empty_diagnostics_for_document(document: &TextDocumentItem) -> PublishDiagnosticsParams {
    PublishDiagnosticsParams {
        uri: document.uri.clone(),
        version: Some(document.version),
        diagnostics: vec![],
    }
}

/// Helper methods.
impl PolarLanguageServer {
    fn add_document(&mut self, doc: TextDocumentItem) {
        self.documents.insert(doc.uri.clone(), doc);
    }

    fn update_document(&mut self, uri: Url, version: i32, text: String) {
        self.documents.entry(uri).and_modify(|doc| {
            doc.version = version;
            doc.text = text;
        });
    }

    fn remove_document(&mut self, uri: &Url) -> PublishDiagnosticsParams {
        let deleted = self.documents.remove(uri).unwrap();
        empty_diagnostics_for_document(&deleted)
    }

    fn send_diagnostics(&self, params: PublishDiagnosticsParams) {
        let this = &JsValue::null();
        let params = &serde_wasm_bindgen::to_value(&params).unwrap();
        self.send_diagnostics_callback.call1(this, params).unwrap();
    }

    fn empty_diagnostics_for_all_documents(&self) -> HashMap<Url, PublishDiagnosticsParams> {
        self.documents
            .values()
            .map(|d| (d.uri.clone(), empty_diagnostics_for_document(d)))
            .collect()
    }

    fn polar_error_to_diagnostic(&self, e: &PolarError) -> Option<PublishDiagnosticsParams> {
        e.context
            .as_ref()
            .and_then(|c| c.source.filename.as_ref())
            .and_then(|uri| Url::parse(uri).ok())
            .map(|uri| {
                let diagnostic = Diagnostic {
                    range: range_from_polar_error(e),
                    severity: Some(DiagnosticSeverity::Error),
                    code: None,
                    code_description: None,
                    source: Some("polar-language-server".to_owned()),
                    message: e.to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                };
                let document = self.documents.get(&uri).unwrap();
                PublishDiagnosticsParams {
                    uri: document.uri.clone(),
                    version: Some(document.version),
                    diagnostics: vec![diagnostic],
                }
            })
    }

    /// Reloads tracked documents into the `KnowledgeBase`, translates errors from `Polar::load`
    /// into diagnostics, and returns a set of diagnostics for publishing.
    ///
    /// NOTE(gj): we currently only receive a single error (pertaining to a single document) at a
    /// time from the core, but we republish 'empty' diagnostics for all other documents in order
    /// to purge stale diagnostics.
    fn reload_kb(&self) -> HashMap<Url, PublishDiagnosticsParams> {
        self.polar.clear_rules();
        let sources = self
            .documents
            .iter()
            .map(|(uri, doc)| Source {
                filename: Some(uri.to_string()),
                src: doc.text.clone(),
            })
            .collect();
        let mut diagnostics = self.empty_diagnostics_for_all_documents();
        if let Err(e) = self.polar.load(sources) {
            if let Some(d) = self.polar_error_to_diagnostic(&e) {
                assert!(diagnostics.insert(d.uri.clone(), d).is_some());
            }
        }
        diagnostics
    }
}

/// Individual LSP notification handlers.
impl PolarLanguageServer {
    fn on_did_open_text_document(&mut self, params: DidOpenTextDocumentParams) {
        self.add_document(params.text_document);
        for (_, diagnostic) in self.reload_kb() {
            self.send_diagnostics(diagnostic);
        }
    }

    fn on_did_change_text_document(&mut self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version },
            content_changes,
        } = params;

        assert_eq!(content_changes.len(), 1);
        // Ensure we receive full -- not incremental -- updates.
        assert!(content_changes[0].range.is_none());

        self.update_document(uri, version, content_changes[0].text.clone());
        for (_, diagnostic) in self.reload_kb() {
            self.send_diagnostics(diagnostic);
        }
    }

    fn on_did_change_watched_files(&mut self, params: DidChangeWatchedFilesParams) {
        for FileEvent { uri, typ } in params.changes {
            assert_eq!(typ, FileChangeType::Deleted); // We only watch for `Deleted` events.
            let params = self.remove_document(&uri);
            self.send_diagnostics(params);
        }
        for (_, diagnostic) in self.reload_kb() {
            self.send_diagnostics(diagnostic);
        }
    }
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;

    use super::*;

    fn new_pls() -> PolarLanguageServer {
        let noop = js_sys::Function::new_with_args("_params", "");
        PolarLanguageServer::new(&noop)
    }

    fn doc_with_no_errors1() -> TextDocumentItem {
        let apple = Url::parse("file:///apple.polar").unwrap();
        TextDocumentItem::new(apple, "polar".to_owned(), 0, "apple();".to_owned())
    }

    fn doc_with_error1() -> TextDocumentItem {
        let apple = Url::parse("file:///apple.polar").unwrap();
        TextDocumentItem::new(apple, "polar".to_owned(), 0, "apple".to_owned())
    }

    fn doc_with_error2() -> TextDocumentItem {
        let apple = Url::parse("file:///banana.polar").unwrap();
        TextDocumentItem::new(apple, "polar".to_owned(), 0, "banana".to_owned())
    }

    fn assert_missing_semicolon_error(params: PublishDiagnosticsParams, doc: TextDocumentItem) {
        assert_eq!(params.uri, doc.uri);
        assert_eq!(params.version.unwrap(), doc.version);
        assert_eq!(params.diagnostics.len(), 1, "{:?}", params.diagnostics);
        let diagnostic = params.diagnostics.into_iter().next().unwrap();
        let expected_message = format!("hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column {column} in file {uri}", column=doc.text.len() + 1, uri=doc.uri);
        assert_eq!(diagnostic.message, expected_message);
    }

    fn assert_no_errors(params: PublishDiagnosticsParams, doc: TextDocumentItem) {
        assert_eq!(params.uri, doc.uri);
        assert_eq!(params.version.unwrap(), doc.version);
        assert_eq!(params.diagnostics.len(), 0);
    }

    #[wasm_bindgen_test]
    fn test_one_document_no_errors() {
        let mut pls = new_pls();
        let doc = doc_with_no_errors1();
        pls.add_document(doc.clone());
        let params = pls.reload_kb();
        assert_eq!(params.len(), 1);
        let params = params.into_values().next().unwrap();
        assert_no_errors(params, doc);
    }

    #[wasm_bindgen_test]
    fn test_one_document_one_error() {
        let mut pls = new_pls();
        let doc = doc_with_error1();
        pls.add_document(doc.clone());
        let params = pls.reload_kb();
        assert_eq!(params.len(), 1);
        let params = params.into_values().next().unwrap();
        assert_missing_semicolon_error(params, doc);
    }

    #[wasm_bindgen_test]
    fn test_two_documents_one_error() {
        let mut pls = new_pls();
        let valid = doc_with_no_errors1();
        let invalid = doc_with_error2();
        pls.add_document(valid.clone());
        pls.add_document(invalid.clone());
        let params = pls.reload_kb();
        assert_eq!(params.len(), 2);
        let mut params = params.into_values();
        // NOTE(gj): we currently surface at most one error per `Polar::load` call, so even if two
        // documents have semicolon errors we'll only publish a single diagnostic.
        assert_missing_semicolon_error(params.next().unwrap(), invalid);
        assert_no_errors(params.next().unwrap(), valid);
    }

    #[wasm_bindgen_test]
    fn test_two_documents_two_errors() {
        let mut pls = new_pls();
        let invalid1 = doc_with_error1();
        let invalid2 = doc_with_error2();
        pls.add_document(invalid1.clone());
        pls.add_document(invalid2.clone());
        let params = pls.reload_kb();
        assert_eq!(params.len(), 2);
        let mut params = params.into_values();
        // NOTE(gj): we currently surface at most one error per `Polar::load` call, so even if two
        // documents have semicolon errors we'll only publish a single diagnostic.
        assert_missing_semicolon_error(params.next().unwrap(), invalid1);
        assert_no_errors(params.next().unwrap(), invalid2);
    }
}
