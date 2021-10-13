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
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct PolarLanguageServer {
    documents: HashMap<Url, TextDocumentItem>,
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
}

/// Individual LSP notification handlers.
impl PolarLanguageServer {
    fn on_did_open_text_document(&mut self, params: DidOpenTextDocumentParams) {
        let DidOpenTextDocumentParams { text_document: doc } = params;
        self.documents.insert(doc.uri.clone(), doc);
        self.send_diagnostics_for_documents();
    }

    fn on_did_change_text_document(&mut self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version },
            content_changes,
        } = params;

        assert_eq!(content_changes.len(), 1);
        assert!(content_changes[0].range.is_none());

        self.documents.entry(uri).and_modify(|previous| {
            previous.version = version;
            previous.text = content_changes[0].text.clone();
        });
        self.send_diagnostics_for_documents();
    }

    fn on_did_change_watched_files(&mut self, params: DidChangeWatchedFilesParams) {
        for FileEvent { uri, typ } in params.changes {
            assert_eq!(typ, FileChangeType::Deleted); // We only watch for `Deleted` events.
            let deleted = self.documents.remove(&uri).unwrap();
            self.clear_diagnostics_for_document(deleted);
        }
        self.send_diagnostics_for_documents();
    }
}
