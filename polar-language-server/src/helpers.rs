use std::collections::BTreeMap;

use lsp_types::{Position, PublishDiagnosticsParams, Range, TextDocumentItem, Url};
use polar_core::diagnostic::Diagnostic;
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
    use polar_core::diagnostic::Range as PolarRange;

    let context = match diagnostic {
        Diagnostic::Error(e) => e.context.as_ref(),
        Diagnostic::Warning(w) => w.context.as_ref(),
    };

    if let Some(PolarRange { start, end }) = context.map(|c| c.range) {
        let start = Position {
            line: start.row as _,
            character: start.column as _,
        };
        let end = Position {
            line: end.row as _,
            character: end.column as _,
        };
        Range { start, end }
    } else {
        Range::default()
    }
}

pub(crate) fn uri_from_polar_diagnostic_context(diagnostic: &Diagnostic) -> Option<Url> {
    let context = match diagnostic {
        Diagnostic::Error(e) => e.context.as_ref(),
        Diagnostic::Warning(w) => w.context.as_ref(),
    };
    if let Some(context) = context {
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
                "source missing filename:\n\t{:?}\n\tDiagnostic: {}",
                context.source, diagnostic
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
