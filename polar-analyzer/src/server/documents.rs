use lsp_types::{
    DeleteFilesParams, Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams,
    DidOpenTextDocumentParams, Position, Range, RenameFilesParams, TextDocumentItem,
};
use polar_core::error::PolarError;
use tracing::debug;

use super::Backend;

impl Backend {
    pub async fn open_document(&self, params: DidOpenTextDocumentParams) -> crate::Result<()> {
        let TextDocumentItem { text, uri, .. } = params.text_document;
        self.try_load_file(text, uri).await;
        Ok(())
    }

    pub async fn edit_document(&self, params: DidChangeTextDocumentParams) -> crate::Result<()> {
        let uri = params.text_document.uri;
        if params.content_changes.len() > 1 {
            anyhow::bail!("not sure how to handle multiple changes to the same file")
        }
        for change in params.content_changes {
            if change.range.is_some() {
                anyhow::bail!("incremental changes are not yet supported")
            }
            let src = change.text;
            self.try_load_file(src, uri.clone()).await;
        }
        Ok(())
    }

    pub async fn rename_files(&self, params: RenameFilesParams) -> crate::Result<()> {
        let polar = self.get_analyzer().await;

        for rename in params.files {
            let old = self.uri_to_string(&rename.old_uri).await;
            let new = self.uri_to_string(&rename.new_uri).await;
            polar.rename(&old, &new)?;
        }
        Ok(())
    }

    pub async fn delete_files(&self, params: DeleteFilesParams) -> crate::Result<()> {
        let polar = self.get_analyzer().await;

        for deletion in params.files {
            let filename = self.uri_to_string(&deletion.uri).await;
            polar.delete(&filename);
        }
        Ok(())
    }

    async fn try_load_file(&self, src: String, uri: lsp_types::Url) {
        let filename = self.uri_to_string(&uri).await;
        let polar = self.get_analyzer().await;
        debug!("Loading: {} as {}", uri, filename);
        let mut diagnostics = vec![];
        if let Err(e) = polar.load(&src, &filename) {
            diagnostics.push(error_to_diagnostic(e))
        }
        for (rule_error, start, end) in polar.get_unused_rules(&filename) {
            let diagnostic = Diagnostic {
                severity: Some(DiagnosticSeverity::Warning),
                message: format!("Rule does not exist: {}", rule_error),
                range: polar
                    .source_map
                    .location_to_range(&filename, start, end)
                    .unwrap(),
                ..Default::default()
            };
            diagnostics.push(diagnostic);
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await
    }
}

fn error_to_diagnostic(error: PolarError) -> Diagnostic {
    let range = error.context.as_ref().map(|ctxt| Range {
        start: Position::new(ctxt.row as u32, ctxt.column as u32),
        end: Position::new(ctxt.row as u32, ctxt.column as u32),
    });
    Diagnostic {
        range: range.unwrap_or_default(),
        severity: Some(DiagnosticSeverity::Error),
        message: error.to_string(),
        source: Some("polar-analzyer".to_string()),
        ..Default::default()
    }
}
