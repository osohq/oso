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
    fn add_doc_with_no_errors(pls: &mut PolarLanguageServer, path: &str) -> TextDocumentItem {
        let doc = doc_with_no_errors(path);
        assert!(pls.upsert_document(doc.clone()).is_none());
        doc
    }

    #[track_caller]
    fn add_doc_with_missing_semicolon(
        pls: &mut PolarLanguageServer,
        path: &str,
    ) -> TextDocumentItem {
        let doc = doc_with_missing_semicolon(path);
        assert!(pls.upsert_document(doc.clone()).is_none());
        doc
    }

    #[track_caller]
    fn update_text(doc: TextDocumentItem, text: &str) -> TextDocumentItem {
        TextDocumentItem::new(doc.uri, doc.language_id, doc.version + 1, text.into())
    }

    #[track_caller]
    fn assert_missing_semicolon_error(diagnostics: &Diagnostics, doc: &TextDocumentItem) {
        let params = diagnostics.get(&doc.uri).unwrap();
        assert_eq!(params.uri, doc.uri);
        assert_eq!(params.version.unwrap(), doc.version);
        assert_eq!(params.diagnostics.len(), 1, "{}", doc.uri.to_string());
        let diagnostic = params.diagnostics.get(0).unwrap();
        let expected_message = format!("hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column {column} in file {uri}", column=doc.text.len() + 1, uri=doc.uri);
        assert_eq!(diagnostic.message, expected_message);
    }

    #[track_caller]
    fn assert_no_errors(diagnostics: &Diagnostics, doc: &TextDocumentItem) {
        let params = diagnostics.get(&doc.uri).unwrap();
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
        assert_no_errors(&diagnostics, &apple);

        // Load a second doc w/ no errors.
        let banana = doc_with_no_errors("banana");
        let diagnostics = pls.on_did_open_text_document(banana.clone());
        assert_eq!(diagnostics.len(), 2);
        assert_no_errors(&diagnostics, &apple);
        assert_no_errors(&diagnostics, &banana);

        // Load a third doc w/ errors.
        let canteloupe = doc_with_missing_semicolon("canteloupe");
        let diagnostics = pls.on_did_open_text_document(canteloupe.clone());
        assert_eq!(diagnostics.len(), 3);
        assert_no_errors(&diagnostics, &apple);
        assert_no_errors(&diagnostics, &banana);
        assert_missing_semicolon_error(&diagnostics, &canteloupe);

        // Load a fourth doc w/ errors.
        let date = doc_with_missing_semicolon("date");
        let diagnostics = pls.on_did_open_text_document(date.clone());
        assert_eq!(diagnostics.len(), 4);
        assert_no_errors(&diagnostics, &apple);
        assert_no_errors(&diagnostics, &banana);
        assert_missing_semicolon_error(&diagnostics, &canteloupe);
        // NOTE(gj): we currently surface at most one error per `Polar::load` call, so even if two
        // documents have semicolon errors we'll only publish a single diagnostic.
        assert_no_errors(&diagnostics, &date);

        // Load a fifth doc w/ no errors.
        let elderberry = doc_with_no_errors("elderberry");
        let diagnostics = pls.on_did_open_text_document(elderberry.clone());
        assert_eq!(diagnostics.len(), 5);
        assert_no_errors(&diagnostics, &apple);
        assert_no_errors(&diagnostics, &banana);
        assert_missing_semicolon_error(&diagnostics, &canteloupe);
        // NOTE(gj): we currently surface at most one error per `Polar::load` call, so even if two
        // documents have semicolon errors we'll only publish a single diagnostic.
        assert_no_errors(&diagnostics, &date);
        assert_no_errors(&diagnostics, &elderberry);
    }

    #[wasm_bindgen_test]
    fn test_on_did_change_text_document() {
        let mut pls = new_pls();

        // 'Change' untracked doc w/ no errors.
        let apple0 = doc_with_no_errors("apple");
        let diagnostics0 = pls.on_did_change_text_document(apple0.clone());
        assert_eq!(diagnostics0.len(), 1);
        assert_no_errors(&diagnostics0, &apple0);

        // Change tracked doc w/o introducing an error.
        let apple1 = update_text(apple0, "pie();");
        let diagnostics1 = pls.on_did_change_text_document(apple1.clone());
        assert_eq!(diagnostics1.len(), 1);
        assert_no_errors(&diagnostics1, &apple1);

        // Change tracked doc, introducing an error.
        let apple2 = update_text(apple1, "pie()");
        let diagnostics2 = pls.on_did_change_text_document(apple2.clone());
        assert_eq!(diagnostics2.len(), 1);
        assert_missing_semicolon_error(&diagnostics2, &apple2);

        // 'Change' untracked doc, introducing a second error.
        let banana0 = doc_with_missing_semicolon("banana");
        let diagnostics3 = pls.on_did_change_text_document(banana0.clone());
        assert_eq!(diagnostics3.len(), 2);
        assert_missing_semicolon_error(&diagnostics3, &apple2);
        // NOTE(gj): we currently surface at most one error per `Polar::load` call, so even if two
        // documents have semicolon errors we'll only publish a single diagnostic.
        assert_no_errors(&diagnostics3, &banana0);

        // Change tracked doc, fixing an error.
        let apple3 = update_text(apple2, "pie();");
        let diagnostics4 = pls.on_did_change_text_document(apple3.clone());
        assert_eq!(diagnostics4.len(), 2);
        assert_no_errors(&diagnostics4, &apple3);
        assert_missing_semicolon_error(&diagnostics4, &banana0);

        // Change tracked doc, fixing the last error.
        let banana1 = update_text(banana0, "split();");
        let diagnostics5 = pls.on_did_change_text_document(banana1.clone());
        assert_eq!(diagnostics5.len(), 2);
        assert_no_errors(&diagnostics5, &apple3);
        assert_no_errors(&diagnostics5, &banana1);
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
        let apple2 = add_doc_with_no_errors(&mut pls, "apple");
        let events2 = vec![FileEvent::new(apple2.uri.clone(), FileChangeType::Deleted)];
        let diagnostics2 = pls.on_did_change_watched_files(events2);
        assert_eq!(diagnostics2.len(), 1);
        assert_no_errors(&diagnostics2, &apple2);
        assert!(pls.documents.is_empty());

        // Deleting tracked doc w/ error.
        let apple3 = add_doc_with_missing_semicolon(&mut pls, "apple");
        let events3 = vec![FileEvent::new(apple3.uri.clone(), FileChangeType::Deleted)];
        let diagnostics3 = pls.on_did_change_watched_files(events3);
        assert_eq!(diagnostics3.len(), 1);
        assert_no_errors(&diagnostics3, &apple3);
        assert!(pls.documents.is_empty());

        // Deleting tracked doc w/o error; doc w/o error remains.
        let apple4 = add_doc_with_no_errors(&mut pls, "apple");
        let banana4 = add_doc_with_no_errors(&mut pls, "banana");
        let events4 = vec![FileEvent::new(apple4.uri.clone(), FileChangeType::Deleted)];
        let diagnostics4 = pls.on_did_change_watched_files(events4);
        assert_eq!(diagnostics4.len(), 2);
        assert_no_errors(&diagnostics4, &apple4);
        assert_no_errors(&diagnostics4, &banana4);
        assert!(pls.remove_document(&banana4.uri).is_some());
        assert!(pls.documents.is_empty());

        // Deleting tracked doc w/ error; doc w/o error remains.
        let apple5 = add_doc_with_missing_semicolon(&mut pls, "apple");
        let banana5 = add_doc_with_no_errors(&mut pls, "banana");
        let events5 = vec![FileEvent::new(apple5.uri.clone(), FileChangeType::Deleted)];
        let diagnostics5 = pls.on_did_change_watched_files(events5);
        assert_eq!(diagnostics5.len(), 2);
        assert_no_errors(&diagnostics5, &apple5);
        assert_no_errors(&diagnostics5, &banana5);
        assert!(pls.remove_document(&banana5.uri).is_some());
        assert!(pls.documents.is_empty());

        // Deleting tracked doc w/o error; doc w/ error remains.
        let apple6 = add_doc_with_no_errors(&mut pls, "apple");
        let banana6 = add_doc_with_missing_semicolon(&mut pls, "banana");
        let events6 = vec![FileEvent::new(apple6.uri.clone(), FileChangeType::Deleted)];
        let diagnostics6 = pls.on_did_change_watched_files(events6);
        assert_eq!(diagnostics6.len(), 2);
        assert_no_errors(&diagnostics6, &apple6);
        assert_missing_semicolon_error(&diagnostics6, &banana6);
        assert!(pls.remove_document(&banana6.uri).is_some());
        assert!(pls.documents.is_empty());

        // Deleting tracked doc w/ error; doc w/ error remains.
        let apple7 = add_doc_with_missing_semicolon(&mut pls, "apple");
        let banana7 = add_doc_with_missing_semicolon(&mut pls, "banana");
        let events7 = vec![FileEvent::new(apple7.uri.clone(), FileChangeType::Deleted)];
        let diagnostics7 = pls.on_did_change_watched_files(events7);
        assert_eq!(diagnostics7.len(), 2);
        assert_no_errors(&diagnostics7, &apple7);
        assert_missing_semicolon_error(&diagnostics7, &banana7);
        assert!(pls.remove_document(&banana7.uri).is_some());
        assert!(pls.documents.is_empty());

        // Deleting multiple docs at once.
        let apple8 = add_doc_with_missing_semicolon(&mut pls, "apple");
        let banana8 = add_doc_with_missing_semicolon(&mut pls, "banana");
        let canteloupe8 = add_doc_with_missing_semicolon(&mut pls, "canteloupe");
        let date8 = add_doc_with_no_errors(&mut pls, "date");
        let elderberry8 = add_doc_with_no_errors(&mut pls, "elderberry");
        let fig8 = add_doc_with_no_errors(&mut pls, "fig");
        let events8 = vec![
            FileEvent::new(apple8.uri.clone(), FileChangeType::Deleted),
            FileEvent::new(banana8.uri.clone(), FileChangeType::Deleted),
            FileEvent::new(date8.uri.clone(), FileChangeType::Deleted),
            FileEvent::new(elderberry8.uri.clone(), FileChangeType::Deleted),
        ];
        let diagnostics8 = pls.on_did_change_watched_files(events8);
        assert_eq!(diagnostics8.len(), 6);
        assert_no_errors(&diagnostics8, &apple8);
        assert_no_errors(&diagnostics8, &banana8);
        assert_missing_semicolon_error(&diagnostics8, &canteloupe8);
        assert_no_errors(&diagnostics8, &date8);
        assert_no_errors(&diagnostics8, &elderberry8);
        assert_no_errors(&diagnostics8, &fig8);
        assert!(pls.remove_document(&canteloupe8.uri).is_some());
        assert!(pls.remove_document(&fig8.uri).is_some());
        assert!(pls.documents.is_empty());

        // Deleting directories containing Polar files.
        let apple9 = add_doc_with_missing_semicolon(&mut pls, "apple");
        let banana9 = add_doc_with_no_errors(&mut pls, "a/b/banana");
        let calabash9 = add_doc_with_no_errors(&mut pls, "a/b/c/ca/calabash");
        let canteloupe9 = add_doc_with_no_errors(&mut pls, "a/b/c/ca/canteloupe");
        let cherry9 = add_doc_with_no_errors(&mut pls, "a/b/c/ch/cherry");
        let date9 = add_doc_with_no_errors(&mut pls, "a/b/c/d/date");
        let grape9 = add_doc_with_no_errors(&mut pls, "a/b/c/d/e/f/g/grape");
        let grapefruit9 = add_doc_with_no_errors(&mut pls, "a/b/c/d/e/f/g/grapefruit");

        // Deleting a deeply nested directory.
        let d_dir = Url::parse(date9.uri.as_str().strip_suffix("/date.polar").unwrap()).unwrap();
        let events9a = vec![FileEvent::new(d_dir, FileChangeType::Deleted)];
        assert_eq!(pls.documents.len(), 8);
        let diagnostics9a = pls.on_did_change_watched_files(events9a);
        assert_eq!(diagnostics9a.len(), 8);
        assert_missing_semicolon_error(&diagnostics9a, &apple9);
        assert_no_errors(&diagnostics9a, &banana9);
        assert_no_errors(&diagnostics9a, &calabash9);
        assert_no_errors(&diagnostics9a, &canteloupe9);
        assert_no_errors(&diagnostics9a, &cherry9);
        assert_no_errors(&diagnostics9a, &date9);
        assert_no_errors(&diagnostics9a, &grape9);
        assert_no_errors(&diagnostics9a, &grapefruit9);
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
        assert_missing_semicolon_error(&diagnostics9b, &apple9);
        assert_no_errors(&diagnostics9b, &banana9);
        assert_no_errors(&diagnostics9b, &calabash9);
        assert_no_errors(&diagnostics9b, &canteloupe9);
        assert_no_errors(&diagnostics9b, &cherry9);
        assert_eq!(pls.documents.len(), 2);

        // Deleting a top-level directory.
        let a_dir = banana9.uri.as_str().strip_suffix("/b/banana.polar");
        let a_dir = Url::parse(a_dir.unwrap()).unwrap();
        let events9c = vec![FileEvent::new(a_dir, FileChangeType::Deleted)];
        assert_eq!(pls.documents.len(), 2);
        let diagnostics9c = pls.on_did_change_watched_files(events9c);
        assert_eq!(diagnostics9c.len(), 2);
        assert_missing_semicolon_error(&diagnostics9c, &apple9);
        assert_no_errors(&diagnostics9c, &banana9);
        assert_eq!(pls.documents.len(), 1);
        assert!(pls.remove_document(&apple9.uri).is_some());
        assert!(pls.documents.is_empty());
    }
}
