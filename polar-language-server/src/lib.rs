use std::collections::HashMap;

use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, Notification},
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    Position, PublishDiagnosticsParams, Range, TextDocumentItem, Url,
    VersionedTextDocumentIdentifier,
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
    send_diagnostics: js_sys::Function,
}

#[wasm_bindgen]
impl PolarLanguageServer {
    #[wasm_bindgen(constructor)]
    pub fn new(send_diagnostics: &js_sys::Function) -> Self {
        console_error_panic_hook::set_once();

        Self {
            documents: HashMap::new(),
            send_diagnostics: send_diagnostics.clone(),
        }
    }

    #[wasm_bindgen(js_class = PolarLanguageServer, js_name = onNotification)]
    pub fn on_notification(&mut self, method: &str, params: JsValue) {
        match method {
            DidOpenTextDocument::METHOD => {
                self.on_did_open_text_document(serde_wasm_bindgen::from_value(params).unwrap())
            }
            DidChangeTextDocument::METHOD => {
                self.on_did_change_text_document(serde_wasm_bindgen::from_value(params).unwrap())
            }
            _ => {
                log(&format!(
                    "[WASM] on_notification\n\t{} => {:?}",
                    method, params
                ));
            }
        }
    }

    #[wasm_bindgen(js_class = PolarLanguageServer, js_name = onRequest)]
    pub fn on_request(&self, method: &str, params: JsValue) {
        log(&format!("[WASM on_request] {} => {:?}", method, params));
    }

    fn send_diagnostics_to_js(&self) {
        for document in self.documents.values() {
            let first_line = document.text.split('\n').next().unwrap();
            if first_line.is_empty() {
                continue;
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
            let params = PublishDiagnosticsParams {
                uri: document.uri.clone(),
                version: Some(document.version),
                diagnostics: vec![diagnostic],
            };
            self.send_diagnostics
                .call1(
                    &JsValue::null(),
                    &serde_wasm_bindgen::to_value(&params).unwrap(),
                )
                .unwrap();
        }
    }
}

/// Individual LSP notification handlers.
impl PolarLanguageServer {
    fn on_did_open_text_document(&mut self, params: DidOpenTextDocumentParams) {
        // log(&format!(
        //     "[WASM] on_did_open_text_document\n\tloaded: {}\n\ttotal documents loaded: {}",
        //     params.text_document.uri,
        //     self.documents.len() + 1
        // ));

        self.documents
            .insert(params.text_document.uri.clone(), params.text_document);
        self.send_diagnostics_to_js();
    }

    fn on_did_change_text_document(&mut self, params: DidChangeTextDocumentParams) {
        // log(&format!(
        //     "[WASM] on_did_change_text_document\n\tchanged: {}",
        //     params.text_document.uri,
        // ));

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
        self.send_diagnostics_to_js();
    }
}
