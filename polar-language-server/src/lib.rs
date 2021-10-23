use std::collections::BTreeMap;

use lsp_types::{
    notification::{
        DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument,
        DidSaveTextDocument, Initialized, Notification,
    },
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidOpenTextDocumentParams, FileChangeType, FileEvent, Position, PublishDiagnosticsParams,
    Range, TextDocumentItem, Url, VersionedTextDocumentIdentifier,
};
use polar_core::{
    error::{PolarError, PolarResult},
    polar::Polar,
    sources::Source,
};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
}

fn log(s: &str) {
    #[allow(unused_unsafe)]
    unsafe {
        console_log(&("[pls] ".to_owned() + s))
    }
}

#[wasm_bindgen]
pub struct PolarLanguageServer {
    documents: BTreeMap<Url, TextDocumentItem>,
    polar: Polar,
    send_diagnostics_callback: js_sys::Function,
}

type Diagnostics = BTreeMap<Url, PublishDiagnosticsParams>;

#[must_use]
fn range_from_polar_error_context(PolarError { context: c, .. }: &PolarError) -> Range {
    let (line, character) = c.as_ref().map_or((0, 0), |c| (c.row as _, c.column as _));
    Range {
        start: Position { line, character },
        end: Position { line, character },
    }
}

#[must_use]
fn uri_from_polar_error_context(e: &PolarError) -> Option<Url> {
    if let Some(context) = e.context.as_ref() {
        if let Some(filename) = context.source.filename.as_ref() {
            match Url::parse(filename) {
                Ok(uri) => return Some(uri),
                Err(err) => {
                    log(&format!(
                        "Url::parse error: {}\n\tFilename: {}\n\tError: {}",
                        err, filename, e
                    ));
                }
            }
        } else {
            log(&format!(
                "source missing filename:\n\t{:?}\n\tError: {}",
                context.source, e
            ));
        }
    } else {
        log(&format!("missing error context:\n\t{:?}", e));
    }
    None
}

/// Public API exposed via WASM.
#[wasm_bindgen]
impl PolarLanguageServer {
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new(send_diagnostics_callback: &js_sys::Function) -> Self {
        console_error_panic_hook::set_once();

        Self {
            documents: BTreeMap::new(),
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
                let DidOpenTextDocumentParams { text_document } = from_value(params).unwrap();
                let diagnostics = self.on_did_open_text_document(text_document);
                diagnostics.values().for_each(|d| self.send_diagnostics(d));
            }
            DidChangeTextDocument::METHOD => {
                let params: DidChangeTextDocumentParams = from_value(params).unwrap();

                // Ensure we receive full -- not incremental -- updates.
                assert_eq!(params.content_changes.len(), 1);
                let change = params.content_changes.into_iter().next().unwrap();
                assert!(change.range.is_none());

                let VersionedTextDocumentIdentifier { uri, version } = params.text_document;
                let updated_doc = TextDocumentItem::new(uri, "polar".into(), version, change.text);
                let diagnostics = self.on_did_change_text_document(updated_doc);
                diagnostics.values().for_each(|d| self.send_diagnostics(d));
            }
            DidChangeWatchedFiles::METHOD => {
                self.on_did_change_watched_files(from_value(params).unwrap())
            }
            // We don't care when a document is saved -- we already have the updated state thanks
            // to `DidChangeTextDocument`.
            DidSaveTextDocument::METHOD => (),
            // We don't care when a document is closed -- we care about all Polar files in a
            // workspace folder regardless of which ones remain open.
            DidCloseTextDocument::METHOD => (),
            // Nothing to do when we receive the `Initialized` notification.
            Initialized::METHOD => (),
            _ => log(&format!("on_notification {} {:?}", method, params)),
        }
    }
}

#[must_use]
fn empty_diagnostics_for_document(document: &TextDocumentItem) -> PublishDiagnosticsParams {
    PublishDiagnosticsParams {
        uri: document.uri.clone(),
        version: Some(document.version),
        diagnostics: vec![],
    }
}

/// Helper methods.
impl PolarLanguageServer {
    #[must_use]
    fn upsert_document(&mut self, doc: TextDocumentItem) -> Option<TextDocumentItem> {
        self.documents.insert(doc.uri.clone(), doc)
    }

    #[must_use]
    fn remove_document(&mut self, uri: &Url) -> Option<TextDocumentItem> {
        self.documents.remove(uri)
    }

    fn send_diagnostics(&self, params: &PublishDiagnosticsParams) {
        let this = &JsValue::null();
        let params = &to_value(params).unwrap();
        if let Err(e) = self.send_diagnostics_callback.call1(this, params) {
            log(&format!(
                "send_diagnostics params:\n\t{:?}\n\tJS error: {:?}",
                params, e
            ));
        }
    }

    #[must_use]
    fn empty_diagnostics_for_all_documents(&self) -> Diagnostics {
        self.documents
            .values()
            .map(|d| (d.uri.clone(), empty_diagnostics_for_document(d)))
            .collect()
    }

    #[must_use]
    fn document_from_polar_error_context(&self, e: &PolarError) -> Option<&TextDocumentItem> {
        uri_from_polar_error_context(e).and_then(|uri| {
            if let Some(document) = self.documents.get(&uri) {
                Some(document)
            } else {
                let tracked_docs = self.documents.keys().map(ToString::to_string);
                let tracked_docs = tracked_docs.collect::<Vec<_>>().join(", ");
                log(&format!(
                    "untracked document: {}\n\tTracked documents: {}\n\tError: {}",
                    uri, tracked_docs, e
                ));
                None
            }
        })
    }

    #[must_use]
    fn diagnostic_from_polar_error(&self, e: &PolarError) -> Option<PublishDiagnosticsParams> {
        self.document_from_polar_error_context(e).map(|d| {
            let diagnostic = Diagnostic {
                range: range_from_polar_error_context(e),
                severity: Some(DiagnosticSeverity::Error),
                source: Some("polar-language-server".to_owned()),
                message: e.to_string(),
                ..Default::default()
            };
            PublishDiagnosticsParams {
                uri: d.uri.clone(),
                version: Some(d.version),
                diagnostics: vec![diagnostic],
            }
        })
    }

    /// Turn tracked documents into a set of Polar `Source` structs for `Polar::load`.
    #[must_use]
    fn documents_to_polar_sources(&self) -> Vec<Source> {
        self.documents
            .values()
            .map(|doc| Source {
                filename: Some(doc.uri.to_string()),
                src: doc.text.clone(),
            })
            .collect()
    }

    fn load_documents(&self) -> PolarResult<()> {
        self.polar.load(self.documents_to_polar_sources())
    }

    /// Reloads tracked documents into the `KnowledgeBase`, translates errors from `Polar::load`
    /// into diagnostics, and returns a set of diagnostics for publishing.
    ///
    /// NOTE(gj): we currently only receive a single error (pertaining to a single document) at a
    /// time from the core, but we republish 'empty' diagnostics for all other documents in order
    /// to purge stale diagnostics.
    #[must_use]
    fn reload_kb(&self) -> Diagnostics {
        self.polar.clear_rules();
        let mut diagnostics = self.empty_diagnostics_for_all_documents();
        if let Err(e) = self.load_documents() {
            if let Some(d) = self.diagnostic_from_polar_error(&e) {
                // NOTE(gj): this assertion should never fail b/c we should only get Polar errors
                // for documents we load into the KB and this `diagnostics` map contains an (empty)
                // entry for every document we load into the KB.
                assert!(diagnostics.insert(d.uri.clone(), d).is_some());
            }
        }
        diagnostics
    }
}

/// Individual LSP notification handlers.
impl PolarLanguageServer {
    #[must_use]
    fn on_did_open_text_document(&mut self, doc: TextDocumentItem) -> Diagnostics {
        if let Some(TextDocumentItem { uri, .. }) = self.upsert_document(doc) {
            log(&format!("reopened tracked document {}", uri));
        }
        self.reload_kb()
    }

    #[must_use]
    fn on_did_change_text_document(&mut self, doc: TextDocumentItem) -> Diagnostics {
        let uri = doc.uri.clone();
        if self.upsert_document(doc).is_none() {
            log(&format!("updated untracked document {}", uri));
        }
        self.reload_kb()
    }

    fn on_did_change_watched_files(&mut self, params: DidChangeWatchedFilesParams) {
        for FileEvent { uri, typ } in params.changes {
            assert_eq!(typ, FileChangeType::Deleted); // We only watch for `Deleted` events.
            if let Some(removed) = self.remove_document(&uri) {
                self.send_diagnostics(&empty_diagnostics_for_document(&removed));
            } else {
                log(&format!("cannot remove untracked document {}", uri));
            }
        }
        for (_, diagnostic) in self.reload_kb() {
            self.send_diagnostics(&diagnostic);
        }
    }
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;

    use super::*;

    #[track_caller]
    fn new_pls() -> PolarLanguageServer {
        let noop = js_sys::Function::new_with_args("_params", "");
        let pls = PolarLanguageServer::new(&noop);
        assert!(pls.reload_kb().is_empty());
        pls
    }

    #[track_caller]
    fn polar_uri(name: &str) -> Url {
        Url::parse(&format!("file:///{}.polar", name)).unwrap()
    }

    #[track_caller]
    fn polar_doc(name: &str, contents: String) -> TextDocumentItem {
        TextDocumentItem::new(polar_uri(name), "polar".to_owned(), 0, contents)
    }

    #[track_caller]
    fn doc_with_no_errors(name: &str) -> TextDocumentItem {
        polar_doc(name, format!("{}();", name))
    }

    #[track_caller]
    fn doc_with_missing_semicolon(name: &str) -> TextDocumentItem {
        polar_doc(name, format!("{}()", name))
    }

    #[track_caller]
    fn update_text(doc: TextDocumentItem, text: &str) -> TextDocumentItem {
        TextDocumentItem::new(doc.uri, doc.language_id, doc.version + 1, text.into())
    }

    #[track_caller]
    fn assert_missing_semicolon_error(params: &PublishDiagnosticsParams, doc: &TextDocumentItem) {
        assert_eq!(params.uri, doc.uri);
        assert_eq!(params.version.unwrap(), doc.version);
        assert_eq!(params.diagnostics.len(), 1, "{}", doc.uri.to_string());
        let diagnostic = params.diagnostics.get(0).unwrap();
        let expected_message = format!("hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column {column} in file {uri}", column=doc.text.len() + 1, uri=doc.uri);
        assert_eq!(diagnostic.message, expected_message);
    }

    #[track_caller]
    fn assert_no_errors(params: &PublishDiagnosticsParams, doc: &TextDocumentItem) {
        assert_eq!(params.uri, doc.uri);
        assert_eq!(params.version.unwrap(), doc.version);
        assert!(params.diagnostics.is_empty(), "{:?}", params.diagnostics);
    }

    #[wasm_bindgen_test]
    fn test_on_did_open_text_document() {
        let mut pls = new_pls();

        // Load a single doc w/ no errors.
        let apple = doc_with_no_errors("apple");
        let diagnostics = pls.on_did_open_text_document(apple.clone());
        assert_eq!(diagnostics.len(), 1);
        let apple_diagnostics = diagnostics.get(&apple.uri).unwrap();
        assert_no_errors(apple_diagnostics, &apple);

        // Load a second doc w/ no errors.
        let banana = doc_with_no_errors("banana");
        let diagnostics = pls.on_did_open_text_document(banana.clone());
        assert_eq!(diagnostics.len(), 2);
        let apple_diagnostics = diagnostics.get(&apple.uri).unwrap();
        let banana_diagnostics = diagnostics.get(&banana.uri).unwrap();
        assert_no_errors(apple_diagnostics, &apple);
        assert_no_errors(banana_diagnostics, &banana);

        // Load a third doc w/ errors.
        let canteloupe = doc_with_missing_semicolon("canteloupe");
        let diagnostics = pls.on_did_open_text_document(canteloupe.clone());
        assert_eq!(diagnostics.len(), 3);
        let apple_diagnostics = diagnostics.get(&apple.uri).unwrap();
        let banana_diagnostics = diagnostics.get(&banana.uri).unwrap();
        let canteloupe_diagnostics = diagnostics.get(&canteloupe.uri).unwrap();
        assert_no_errors(apple_diagnostics, &apple);
        assert_no_errors(banana_diagnostics, &banana);
        assert_missing_semicolon_error(canteloupe_diagnostics, &canteloupe);

        // Load a fourth doc w/ errors.
        let date = doc_with_missing_semicolon("date");
        let diagnostics = pls.on_did_open_text_document(date.clone());
        assert_eq!(diagnostics.len(), 4);
        let apple_diagnostics = diagnostics.get(&apple.uri).unwrap();
        let banana_diagnostics = diagnostics.get(&banana.uri).unwrap();
        let canteloupe_diagnostics = diagnostics.get(&canteloupe.uri).unwrap();
        let date_diagnostics = diagnostics.get(&date.uri).unwrap();
        assert_no_errors(apple_diagnostics, &apple);
        assert_no_errors(banana_diagnostics, &banana);
        assert_missing_semicolon_error(canteloupe_diagnostics, &canteloupe);
        // NOTE(gj): we currently surface at most one error per `Polar::load` call, so even if two
        // documents have semicolon errors we'll only publish a single diagnostic.
        assert_no_errors(date_diagnostics, &date);

        // Load a fifth doc w/ no errors.
        let elderberry = doc_with_no_errors("elderberry");
        let diagnostics = pls.on_did_open_text_document(elderberry.clone());
        assert_eq!(diagnostics.len(), 5);
        let apple_diagnostics = diagnostics.get(&apple.uri).unwrap();
        let banana_diagnostics = diagnostics.get(&banana.uri).unwrap();
        let canteloupe_diagnostics = diagnostics.get(&canteloupe.uri).unwrap();
        let date_diagnostics = diagnostics.get(&date.uri).unwrap();
        let elderberry_diagnostics = diagnostics.get(&elderberry.uri).unwrap();
        assert_no_errors(apple_diagnostics, &apple);
        assert_no_errors(banana_diagnostics, &banana);
        assert_missing_semicolon_error(canteloupe_diagnostics, &canteloupe);
        // NOTE(gj): we currently surface at most one error per `Polar::load` call, so even if two
        // documents have semicolon errors we'll only publish a single diagnostic.
        assert_no_errors(date_diagnostics, &date);
        assert_no_errors(elderberry_diagnostics, &elderberry);
    }

    #[wasm_bindgen_test]
    fn test_on_did_change_text_document() {
        let mut pls = new_pls();

        // 'Change' untracked doc w/ no errors.
        let apple0 = doc_with_no_errors("apple");
        let diagnostics0 = pls.on_did_change_text_document(apple0.clone());
        assert_eq!(diagnostics0.len(), 1);
        let apple0_diagnostics = diagnostics0.get(&apple0.uri).unwrap();
        assert_no_errors(apple0_diagnostics, &apple0);

        // Change tracked doc w/o introducing an error.
        let apple1 = update_text(apple0, "pie();");
        let diagnostics1 = pls.on_did_change_text_document(apple1.clone());
        assert_eq!(diagnostics1.len(), 1);
        let apple1_diagnostics = diagnostics1.get(&apple1.uri).unwrap();
        assert_no_errors(apple1_diagnostics, &apple1);

        // Change tracked doc, introducing an error.
        let apple2 = update_text(apple1, "pie()");
        let diagnostics2 = pls.on_did_change_text_document(apple2.clone());
        assert_eq!(diagnostics2.len(), 1);
        let apple2_diagnostics = diagnostics2.get(&apple2.uri).unwrap();
        assert_missing_semicolon_error(apple2_diagnostics, &apple2);

        // 'Change' untracked doc, introducing a second error.
        let banana0 = doc_with_missing_semicolon("banana");
        let diagnostics3 = pls.on_did_change_text_document(banana0.clone());
        assert_eq!(diagnostics3.len(), 2);
        let apple2_diagnostics = diagnostics3.get(&apple2.uri).unwrap();
        let banana0_diagnostics = diagnostics3.get(&banana0.uri).unwrap();
        assert_missing_semicolon_error(apple2_diagnostics, &apple2);
        // NOTE(gj): we currently surface at most one error per `Polar::load` call, so even if two
        // documents have semicolon errors we'll only publish a single diagnostic.
        assert_no_errors(banana0_diagnostics, &banana0);

        // Change tracked doc, fixing an error.
        let apple3 = update_text(apple2, "pie();");
        let diagnostics4 = pls.on_did_change_text_document(apple3.clone());
        assert_eq!(diagnostics4.len(), 2);
        let apple3_diagnostics = diagnostics4.get(&apple3.uri).unwrap();
        let banana0_diagnostics = diagnostics4.get(&banana0.uri).unwrap();
        assert_no_errors(apple3_diagnostics, &apple3);
        assert_missing_semicolon_error(banana0_diagnostics, &banana0);

        // Change tracked doc, fixing the last error.
        let banana1 = update_text(banana0, "split();");
        let diagnostics5 = pls.on_did_change_text_document(banana1.clone());
        assert_eq!(diagnostics5.len(), 2);
        let apple3_diagnostics = diagnostics5.get(&apple3.uri).unwrap();
        let banana1_diagnostics = diagnostics5.get(&banana1.uri).unwrap();
        assert_no_errors(apple3_diagnostics, &apple3);
        assert_no_errors(banana1_diagnostics, &banana1);
    }
}
