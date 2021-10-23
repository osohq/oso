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

#[cfg(not(test))]
fn log(s: &str) {
    #[allow(unused_unsafe)]
    unsafe {
        console_log(&("[pls] ".to_owned() + s))
    }
}

#[cfg(test)]
fn log(_: &str) {}

type Documents = BTreeMap<Url, TextDocumentItem>;

#[wasm_bindgen]
pub struct PolarLanguageServer {
    documents: Documents,
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
                let DidChangeWatchedFilesParams { changes } = from_value(params).unwrap();
                let diagnostics = self.on_did_change_watched_files(changes);
                diagnostics.values().for_each(|d| self.send_diagnostics(d));
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
                    "untracked doc: {}\n\tTracked: {}\n\tError: {}",
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
            log(&format!("reopened tracked doc: {}", uri));
        }
        self.reload_kb()
    }

    #[must_use]
    fn on_did_change_text_document(&mut self, doc: TextDocumentItem) -> Diagnostics {
        let uri = doc.uri.clone();
        if self.upsert_document(doc).is_none() {
            log(&format!("updated untracked doc: {}", uri));
        }
        self.reload_kb()
    }

    #[must_use]
    fn on_did_change_watched_files(&mut self, changes: Vec<FileEvent>) -> Diagnostics {
        let mut diagnostics = Diagnostics::new();

        for FileEvent { uri, typ } in changes {
            assert_eq!(typ, FileChangeType::Deleted); // We only watch for `Deleted` events.
            let mut msg = format!("deleting URI: {}", uri);

            if let Some(removed) = self.remove_document(&uri) {
                let empty_diagnostics = empty_diagnostics_for_document(&removed);
                if diagnostics.insert(uri, empty_diagnostics).is_some() {
                    msg += "\n\tduplicate watched file event";
                }
            } else {
                msg += "\n\tchecking if URI is dir";
                let docs = self.documents.clone().into_iter();
                let (removed, retained): (Documents, Documents) = docs.partition(|(doc_uri, _)| {
                    let maybe_segments = uri.path_segments().zip(doc_uri.path_segments());
                    // If all path segments match between dir & doc, dir contains doc and doc
                    // should be removed.
                    maybe_segments.map_or(false, |(a, b)| a.zip(b).all(|(x, y)| x == y))
                });
                if removed.is_empty() {
                    msg += "\n\tcannot remove untracked doc";
                } else {
                    for (uri, doc) in removed {
                        msg += &format!("\n\t\tremoving dir member: {}", uri);
                        let empty_diagnostics = empty_diagnostics_for_document(&doc);
                        if diagnostics.insert(uri, empty_diagnostics).is_some() {
                            msg += "\n\t\tduplicate watched file event";
                        }
                    }
                    self.documents = retained;
                }
            }
            log(&msg);
        }

        diagnostics.append(&mut self.reload_kb());
        diagnostics
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
    fn polar_uri(path: &str) -> Url {
        Url::parse(&format!("file:///{}.polar", path)).unwrap()
    }

    #[track_caller]
    fn polar_doc(path: &str, contents: String) -> TextDocumentItem {
        TextDocumentItem::new(polar_uri(path), "polar".to_owned(), 0, contents)
    }

    #[track_caller]
    fn doc_with_no_errors(path: &str) -> TextDocumentItem {
        let file_name = path.split('/').last().unwrap();
        polar_doc(path, format!("{}();", file_name))
    }

    #[track_caller]
    fn doc_with_missing_semicolon(path: &str) -> TextDocumentItem {
        let file_name = path.split('/').last().unwrap();
        polar_doc(path, format!("{}()", file_name))
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

    #[wasm_bindgen_test]
    fn test_on_did_change_watched_files() {
        let mut pls = new_pls();

        // Empty event has no effect.
        let diagnostics0 = pls.on_did_change_watched_files(vec![]);
        assert!(diagnostics0.is_empty());
        assert!(pls.documents.is_empty());

        // Deleting untracked doc has no effect.
        let events1 = vec![FileEvent::new(polar_uri("apple"), FileChangeType::Deleted)];
        let diagnostics1 = pls.on_did_change_watched_files(events1);
        assert!(diagnostics1.is_empty());
        assert!(pls.documents.is_empty());

        // Deleting tracked doc w/o error.
        let apple2 = doc_with_no_errors("apple");
        assert!(pls.upsert_document(apple2.clone()).is_none());
        let events2 = vec![FileEvent::new(apple2.uri.clone(), FileChangeType::Deleted)];
        let diagnostics2 = pls.on_did_change_watched_files(events2);
        assert_eq!(diagnostics2.len(), 1);
        let apple2_diagnostics = diagnostics2.get(&apple2.uri).unwrap();
        assert_no_errors(apple2_diagnostics, &apple2);
        assert!(pls.documents.is_empty());

        // Deleting tracked doc w/ error.
        let apple3 = doc_with_missing_semicolon("apple");
        assert!(pls.upsert_document(apple3.clone()).is_none());
        let events3 = vec![FileEvent::new(apple3.uri.clone(), FileChangeType::Deleted)];
        let diagnostics3 = pls.on_did_change_watched_files(events3);
        assert_eq!(diagnostics3.len(), 1);
        let apple3_diagnostics = diagnostics3.get(&apple3.uri).unwrap();
        assert_no_errors(apple3_diagnostics, &apple3);
        assert!(pls.documents.is_empty());

        // Deleting tracked doc w/o error; doc w/o error remains.
        let apple4 = doc_with_no_errors("apple");
        let banana4 = doc_with_no_errors("banana");
        assert!(pls.upsert_document(apple4.clone()).is_none());
        assert!(pls.upsert_document(banana4.clone()).is_none());
        let events4 = vec![FileEvent::new(apple4.uri.clone(), FileChangeType::Deleted)];
        let diagnostics4 = pls.on_did_change_watched_files(events4);
        assert_eq!(diagnostics4.len(), 2);
        let apple4_diagnostics = diagnostics4.get(&apple4.uri).unwrap();
        let banana4_diagnostics = diagnostics4.get(&banana4.uri).unwrap();
        assert_no_errors(apple4_diagnostics, &apple4);
        assert_no_errors(banana4_diagnostics, &banana4);
        assert!(pls.remove_document(&banana4.uri).is_some());
        assert!(pls.documents.is_empty());

        // Deleting tracked doc w/ error; doc w/o error remains.
        let apple5 = doc_with_missing_semicolon("apple");
        let banana5 = doc_with_no_errors("banana");
        assert!(pls.upsert_document(apple5.clone()).is_none());
        assert!(pls.upsert_document(banana5.clone()).is_none());
        let events5 = vec![FileEvent::new(apple5.uri.clone(), FileChangeType::Deleted)];
        let diagnostics5 = pls.on_did_change_watched_files(events5);
        assert_eq!(diagnostics5.len(), 2);
        let apple5_diagnostics = diagnostics5.get(&apple5.uri).unwrap();
        let banana5_diagnostics = diagnostics5.get(&banana5.uri).unwrap();
        assert_no_errors(apple5_diagnostics, &apple5);
        assert_no_errors(banana5_diagnostics, &banana5);
        assert!(pls.remove_document(&banana5.uri).is_some());
        assert!(pls.documents.is_empty());

        // Deleting tracked doc w/o error; doc w/ error remains.
        let apple6 = doc_with_no_errors("apple");
        let banana6 = doc_with_missing_semicolon("banana");
        assert!(pls.upsert_document(apple6.clone()).is_none());
        assert!(pls.upsert_document(banana6.clone()).is_none());
        let events6 = vec![FileEvent::new(apple6.uri.clone(), FileChangeType::Deleted)];
        let diagnostics6 = pls.on_did_change_watched_files(events6);
        assert_eq!(diagnostics6.len(), 2);
        let apple6_diagnostics = diagnostics6.get(&apple6.uri).unwrap();
        let banana6_diagnostics = diagnostics6.get(&banana6.uri).unwrap();
        assert_no_errors(apple6_diagnostics, &apple6);
        assert_missing_semicolon_error(banana6_diagnostics, &banana6);
        assert!(pls.remove_document(&banana6.uri).is_some());
        assert!(pls.documents.is_empty());

        // Deleting tracked doc w/ error; doc w/ error remains.
        let apple7 = doc_with_missing_semicolon("apple");
        let banana7 = doc_with_missing_semicolon("banana");
        assert!(pls.upsert_document(apple7.clone()).is_none());
        assert!(pls.upsert_document(banana7.clone()).is_none());
        let events7 = vec![FileEvent::new(apple7.uri.clone(), FileChangeType::Deleted)];
        let diagnostics7 = pls.on_did_change_watched_files(events7);
        assert_eq!(diagnostics7.len(), 2);
        let apple7_diagnostics = diagnostics7.get(&apple7.uri).unwrap();
        let banana7_diagnostics = diagnostics7.get(&banana7.uri).unwrap();
        assert_no_errors(apple7_diagnostics, &apple7);
        assert_missing_semicolon_error(banana7_diagnostics, &banana7);
        assert!(pls.remove_document(&banana7.uri).is_some());
        assert!(pls.documents.is_empty());

        // Deleting multiple docs at once.
        let apple8 = doc_with_missing_semicolon("apple");
        let banana8 = doc_with_missing_semicolon("banana");
        let canteloupe8 = doc_with_missing_semicolon("canteloupe");
        let date8 = doc_with_no_errors("date");
        let elderberry8 = doc_with_no_errors("elderberry");
        let fig8 = doc_with_no_errors("fig");
        assert!(pls.upsert_document(apple8.clone()).is_none());
        assert!(pls.upsert_document(banana8.clone()).is_none());
        assert!(pls.upsert_document(canteloupe8.clone()).is_none());
        assert!(pls.upsert_document(date8.clone()).is_none());
        assert!(pls.upsert_document(elderberry8.clone()).is_none());
        assert!(pls.upsert_document(fig8.clone()).is_none());
        let events8 = vec![
            FileEvent::new(apple8.uri.clone(), FileChangeType::Deleted),
            FileEvent::new(banana8.uri.clone(), FileChangeType::Deleted),
            FileEvent::new(date8.uri.clone(), FileChangeType::Deleted),
            FileEvent::new(elderberry8.uri.clone(), FileChangeType::Deleted),
        ];
        let diagnostics8 = pls.on_did_change_watched_files(events8);
        assert_eq!(diagnostics8.len(), 6);
        let apple8_diagnostics = diagnostics8.get(&apple8.uri).unwrap();
        let banana8_diagnostics = diagnostics8.get(&banana8.uri).unwrap();
        let canteloupe8_diagnostics = diagnostics8.get(&canteloupe8.uri).unwrap();
        let date8_diagnostics = diagnostics8.get(&date8.uri).unwrap();
        let elderberry8_diagnostics = diagnostics8.get(&elderberry8.uri).unwrap();
        let fig8_diagnostics = diagnostics8.get(&fig8.uri).unwrap();
        assert_no_errors(apple8_diagnostics, &apple8);
        assert_no_errors(banana8_diagnostics, &banana8);
        assert_missing_semicolon_error(canteloupe8_diagnostics, &canteloupe8);
        assert_no_errors(date8_diagnostics, &date8);
        assert_no_errors(elderberry8_diagnostics, &elderberry8);
        assert_no_errors(fig8_diagnostics, &fig8);
        assert!(pls.remove_document(&canteloupe8.uri).is_some());
        assert!(pls.remove_document(&fig8.uri).is_some());
        assert!(pls.documents.is_empty());

        // Deleting directories containing Polar files.
        let apple9 = doc_with_missing_semicolon("apple");
        let banana9 = doc_with_no_errors("a/b/banana");
        let calabash9 = doc_with_no_errors("a/b/c/ca/calabash");
        let canteloupe9 = doc_with_no_errors("a/b/c/ca/canteloupe");
        let cherry9 = doc_with_no_errors("a/b/c/ch/cherry");
        let date9 = doc_with_no_errors("a/b/c/d/date");
        let grape9 = doc_with_no_errors("a/b/c/d/e/f/g/grape");
        let grapefruit9 = doc_with_no_errors("a/b/c/d/e/f/g/grapefruit");
        assert!(pls.upsert_document(apple9.clone()).is_none());
        assert!(pls.upsert_document(banana9.clone()).is_none());
        assert!(pls.upsert_document(calabash9.clone()).is_none());
        assert!(pls.upsert_document(canteloupe9.clone()).is_none());
        assert!(pls.upsert_document(cherry9.clone()).is_none());
        assert!(pls.upsert_document(date9.clone()).is_none());
        assert!(pls.upsert_document(grape9.clone()).is_none());
        assert!(pls.upsert_document(grapefruit9.clone()).is_none());

        // Deleting a deeply nested directory.
        let d_dir = Url::parse(date9.uri.as_str().strip_suffix("/date.polar").unwrap()).unwrap();
        let events9a = vec![FileEvent::new(d_dir, FileChangeType::Deleted)];
        assert_eq!(pls.documents.len(), 8);
        let diagnostics9a = pls.on_did_change_watched_files(events9a);
        assert_eq!(diagnostics9a.len(), 8);
        let apple9_diagnostics = diagnostics9a.get(&apple9.uri).unwrap();
        let banana9_diagnostics = diagnostics9a.get(&banana9.uri).unwrap();
        let calabash9_diagnostics = diagnostics9a.get(&calabash9.uri).unwrap();
        let canteloupe9_diagnostics = diagnostics9a.get(&canteloupe9.uri).unwrap();
        let cherry9_diagnostics = diagnostics9a.get(&cherry9.uri).unwrap();
        let date9_diagnostics = diagnostics9a.get(&date9.uri).unwrap();
        let grape9_diagnostics = diagnostics9a.get(&grape9.uri).unwrap();
        let grapefruit9_diagnostics = diagnostics9a.get(&grapefruit9.uri).unwrap();
        assert_missing_semicolon_error(apple9_diagnostics, &apple9);
        assert_no_errors(banana9_diagnostics, &banana9);
        assert_no_errors(calabash9_diagnostics, &calabash9);
        assert_no_errors(canteloupe9_diagnostics, &canteloupe9);
        assert_no_errors(cherry9_diagnostics, &cherry9);
        assert_no_errors(date9_diagnostics, &date9);
        assert_no_errors(grape9_diagnostics, &grape9);
        assert_no_errors(grapefruit9_diagnostics, &grapefruit9);
        assert_eq!(pls.documents.len(), 5);

        // Deleting multiple directories at once.
        let ca_dir = calabash9.uri.as_str().strip_suffix("/calabash.polar");
        let ca_dir = Url::parse(ca_dir.unwrap()).unwrap();
        let ch_dir = cherry9.uri.as_str().strip_suffix("/cherry.polar");
        let ch_dir = Url::parse(ch_dir.unwrap()).unwrap();
        let events9b = vec![
            FileEvent::new(ca_dir, FileChangeType::Deleted),
            FileEvent::new(ch_dir, FileChangeType::Deleted),
        ];
        assert_eq!(pls.documents.len(), 5);
        let diagnostics9b = pls.on_did_change_watched_files(events9b);
        assert_eq!(diagnostics9b.len(), 5);
        let apple9_diagnostics = diagnostics9b.get(&apple9.uri).unwrap();
        let banana9_diagnostics = diagnostics9b.get(&banana9.uri).unwrap();
        let calabash9_diagnostics = diagnostics9b.get(&calabash9.uri).unwrap();
        let canteloupe9_diagnostics = diagnostics9b.get(&canteloupe9.uri).unwrap();
        let cherry9_diagnostics = diagnostics9b.get(&cherry9.uri).unwrap();
        assert_missing_semicolon_error(apple9_diagnostics, &apple9);
        assert_no_errors(banana9_diagnostics, &banana9);
        assert_no_errors(calabash9_diagnostics, &calabash9);
        assert_no_errors(canteloupe9_diagnostics, &canteloupe9);
        assert_no_errors(cherry9_diagnostics, &cherry9);
        assert_eq!(pls.documents.len(), 2);

        // Deleting a top-level directory.
        let a_dir = banana9.uri.as_str().strip_suffix("/b/banana.polar");
        let a_dir = Url::parse(a_dir.unwrap()).unwrap();
        let events9c = vec![FileEvent::new(a_dir, FileChangeType::Deleted)];
        assert_eq!(pls.documents.len(), 2);
        let diagnostics9c = pls.on_did_change_watched_files(events9c);
        assert_eq!(diagnostics9c.len(), 2);
        let apple9_diagnostics = diagnostics9c.get(&apple9.uri).unwrap();
        let banana9_diagnostics = diagnostics9c.get(&banana9.uri).unwrap();
        assert_missing_semicolon_error(apple9_diagnostics, &apple9);
        assert_no_errors(banana9_diagnostics, &banana9);
        assert_eq!(pls.documents.len(), 1);
        assert!(pls.remove_document(&apple9.uri).is_some());
        assert!(pls.documents.is_empty());
    }
}
