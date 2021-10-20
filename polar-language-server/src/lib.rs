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
use polar_core::{polar::Polar, sources::Source};
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

// Temporary helper until we get real errors/warnings from `polar-core` to turn into Diagnostics.
fn diagnostics_for_document(document: &TextDocumentItem) -> Option<PublishDiagnosticsParams> {
    let first_line = document.text.split('\n').next().unwrap();
    if first_line.is_empty() {
        return None;
    }
    let diagnostic = Diagnostic {
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: first_line.len() as u32,
            },
        },
        severity: Some(DiagnosticSeverity::Error),
        code: None,
        code_description: None,
        source: Some("polar-language-server".to_owned()),
        message: first_line.to_owned(),
        related_information: None,
        tags: None,
        data: None,
    };
    Some(PublishDiagnosticsParams {
        uri: document.uri.clone(),
        version: Some(document.version),
        diagnostics: vec![diagnostic],
    })
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

    // TODO(gj): not sure we need this yet.
    #[wasm_bindgen(js_class = PolarLanguageServer, js_name = onRequest)]
    pub fn on_request(&self, method: &str, params: JsValue) {
        log(&format!("[WASM on_request] {} => {:?}", method, params));
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

    fn remove_document(&mut self, uri: &Url) {
        let deleted = self.documents.remove(uri).unwrap();
        self.clear_diagnostics_for_document(deleted);
    }

    fn send_diagnostics(&self, params: PublishDiagnosticsParams) {
        let this = &JsValue::null();
        let params = &serde_wasm_bindgen::to_value(&params).unwrap();
        self.send_diagnostics_callback.call1(this, params).unwrap();
    }

    fn clear_diagnostics_for_document(&self, document: TextDocumentItem) {
        let params = PublishDiagnosticsParams {
            uri: document.uri.clone(),
            version: Some(document.version),
            diagnostics: vec![],
        };
        self.send_diagnostics(params);
    }

    fn send_diagnostics_for_documents(&self) {
        for document in self.documents.values() {
            if let Some(params) = diagnostics_for_document(document) {
                self.send_diagnostics(params);
            }
        }
    }

    // TODO(gj): clear all diagnostics when calling this function? Otherwise what if there are a
    // bunch of diagnostics but then we introduce an unrecoverable ParseError (e.g., missing
    // semicolon) and until we fix the new ParseError all of the old errors will remain regardless
    // of whether we fix them or not... maybe?
    fn reload_kb(&self) {
        self.polar.clear_rules();
        let sources = self
            .documents
            .iter()
            .map(|(uri, doc)| Source {
                filename: Some(uri.to_string()),
                src: doc.text.clone(),
            })
            .collect();
        if let Err(e) = self.polar.load(sources) {
            log(&format!("[WASM] Polar::load error\n\t{}", e));
        } else {
            log("[WASM] Polar::load no errors");
        }

        // TODO(gj): temporary until we turn errors/warnings from Polar::load into diagnostics.
        self.send_diagnostics_for_documents();
    }
}

/// Individual LSP notification handlers.
impl PolarLanguageServer {
    fn on_did_open_text_document(&mut self, params: DidOpenTextDocumentParams) {
        self.add_document(params.text_document);
        self.reload_kb();
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
        self.reload_kb();
    }

    fn on_did_change_watched_files(&mut self, params: DidChangeWatchedFilesParams) {
        for FileEvent { uri, typ } in params.changes {
            assert_eq!(typ, FileChangeType::Deleted); // We only watch for `Deleted` events.
            self.remove_document(&uri);
        }
        self.reload_kb();
    }
}
