use std::collections::{BTreeMap, HashSet};

use lsp_types::{Position, PublishDiagnosticsParams, Range, TextDocumentItem, Url};
use polar_core::diagnostic::Diagnostic;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
}

#[cfg(not(test))]
pub(crate) fn log(s: &str) {
    #[allow(unused_unsafe)]
    unsafe {
        console_log(&("[pls] ".to_owned() + s))
    }
}

#[cfg(test)]
pub(crate) fn log(_: &str) {}

pub(crate) type Documents = BTreeMap<Url, TextDocumentItem>;
pub(crate) type Diagnostics = BTreeMap<Url, PublishDiagnosticsParams>;

pub(crate) fn range_from_polar_diagnostic_context(diagnostic: &Diagnostic) -> Range {
    use polar_core::loc_to_pos;

    diagnostic
        .get_context()
        .map(|context| {
            let (row, column) = loc_to_pos(&context.source.src, context.left);
            let start = Position::new(row as _, column as _);
            let (row, column) = loc_to_pos(&context.source.src, context.right);
            let end = Position::new(row as _, column as _);
            Range { start, end }
        })
        .unwrap_or_default()
}

pub(crate) fn uri_from_polar_diagnostic_context(diagnostic: &Diagnostic) -> Option<Url> {
    if let Some(context) = diagnostic.get_context() {
        if let Some(filename) = context.source.filename.as_ref() {
            match Url::parse(filename) {
                Ok(uri) => return Some(uri),
                Err(err) => {
                    log(&format!(
                        "Url::parse error: {}\n\tFilename: {}\n\tDiagnostic: {}",
                        err, filename, diagnostic
                    ));
                }
            }
        } else {
            log(&format!(
                "Diagnostic source missing filename: {}",
                diagnostic
            ));
        }
    } else {
        log(&format!("missing context:\n\t{:?}", diagnostic));
    }
    None
}

pub(crate) fn empty_diagnostics_for_doc(
    (uri, doc): (&Url, &TextDocumentItem),
) -> (Url, PublishDiagnosticsParams) {
    let params = PublishDiagnosticsParams::new(uri.clone(), vec![], Some(doc.version));
    (uri.clone(), params)
}

#[derive(Default, Serialize)]
pub(crate) struct LspEvent<'a> {
    pub(crate) lsp_method: &'a str,
    pub(crate) lsp_file_extensions: HashSet<String>,
}

pub(crate) fn unique_extensions(uris: &[&Url]) -> HashSet<String> {
    uris.iter()
        .filter_map(|uri| uri.as_str().rsplit_once('.'))
        .map(|(_, suffix)| suffix.into())
        .collect()
}
