mod completion;
mod config;
mod documents;
mod hover;
mod symbols;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use std::sync::Arc;

use log::{error, warn};

pub struct Backend {
    pub client: Client,
    pub analyzer: Arc<RwLock<crate::Polar>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let server_capabilities = config::server_capabilities();

        let initialize_result = lsp_types::InitializeResult {
            capabilities: server_capabilities,
            server_info: Some(lsp_types::ServerInfo {
                name: String::from("rust-analyzer"),
                ..Default::default()
            }),
        };

        Ok(initialize_result)
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::Info, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        let _ = params;
        warn!("Got a workspace/didChangeWorkspaceFolders notification, but it is not implemented");
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let _ = params;
        error!("Got a workspace/symbol request, but it is not implemented");
        Err(tower_lsp::jsonrpc::Error::method_not_found())
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        if let Err(e) = documents::rename_files(&*self.analyzer.read().await, params) {
            error!("{}", e)
        }
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        if let Err(e) = documents::delete_files(&*self.analyzer.read().await, params) {
            error!("{}", e)
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        if let Err(e) = documents::open_document(self, params).await {
            error!("{}", e)
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Err(e) = documents::edit_document(self, params).await {
            error!("{}", e)
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(completion::get_completions(params))
    }

    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        Ok(completion::resolve_completion(params))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        Ok(hover::get_hover(&*self.analyzer.read().await, params))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        Ok(symbols::get_document_symbols(
            &*self.analyzer.read().await,
            params,
        ))
    }
}

pub async fn run_tcp_server(polar: Option<crate::Polar>, port: u32) -> anyhow::Result<()> {
    let (service, messages) = LspService::new(|client| Backend {
        client,
        analyzer: Arc::new(RwLock::new(polar.unwrap_or_default())),
    });

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    let (stream, _) = listener.accept().await?;
    let (read, write) = tokio::io::split(stream);
    let server = Server::new(read, write);

    tokio::select! {
        _ = server.interleave(messages).serve(service) => {},
        _ = tokio::signal::ctrl_c() => {},
    };
    Ok(())
}

pub async fn run_stdio_server(polar: Option<crate::Polar>) -> anyhow::Result<()> {
    let (service, messages) = LspService::new(|client| Backend {
        client,
        analyzer: Arc::new(RwLock::new(polar.unwrap_or_default())),
    });

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    tokio::select! {
        _ = Server::new(stdin, stdout)
            .interleave(messages)
            .serve(service) => {},
        _ = tokio::signal::ctrl_c() => {},
    }
    Ok(())
}
