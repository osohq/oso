mod completion;
mod config;
mod cross_language;
mod documents;
mod hover;
mod symbols;

use lsp_types::*;
use polar_core::terms::{Symbol, Term, Value};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use std::sync::Arc;

use tracing::{debug, error, warn};

pub struct Backend {
    pub client: Client,
    pub analyzer: Arc<RwLock<crate::Polar>>,
    pub root: Arc<RwLock<Option<String>>>,
}

impl Backend {
    pub async fn uri_to_string(&self, uri: &impl ToString) -> String {
        if let Some(root) = self.root.read().await.as_ref() {
            uri.to_string().replace(root, "")
        } else {
            uri.to_string()
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(root) = params.root_uri {
            self.root.write().await.replace(root.to_string() + "/");
        }
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
        tracing::trace!("get workspace symbols");
        Ok(Some(vec![]))
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        if let Err(e) = self.rename_files(params).await {
            error!("{}", e)
        }
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        if let Err(e) = self.delete_files(params).await {
            error!("{}", e)
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        if let Err(e) = self.open_document(params).await {
            error!("{}", e)
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Err(e) = self.edit_document(params).await {
            error!("{}", e)
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let symbols_res = self.get_all_symbols().await;

        match symbols_res {
            Ok(symbols) => {
                debug!("Adding symbols to Polar: {:#?}", symbols);
                let polar = self.analyzer.write().await;
                for class in symbols.classes.into_iter() {
                    polar.inner.register_constant(
                        Symbol(class),
                        Term::new_temporary(Value::Boolean(false)),
                    )
                }
            }
            Err(e) => {
                error!("Couldn't get symbols: {}", e)
            }
        }
        Ok(completion::get_completions(params))
    }

    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        Ok(completion::resolve_completion(params))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        Ok(self.get_hover(params).await)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        Ok(self.get_document_symbols(params).await)
    }
}

pub async fn run_tcp_server(polar: Option<crate::Polar>, port: u32) -> anyhow::Result<()> {
    let analyzer = Arc::new(RwLock::new(polar.unwrap_or_default()));
    loop {
        let (service, messages) = LspService::new(|client| Backend {
            client,
            analyzer: analyzer.clone(),
            root: Default::default(),
        });

        debug!("Waiting for connections on port: {}", port);
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        let (stream, _) = listener.accept().await?;
        let (read, write) = tokio::io::split(stream);
        let server = Server::new(read, write);

        tokio::select! {
            _ = server.interleave(messages).serve(service) => {},
            _ = tokio::signal::ctrl_c() => break,
        };
    }
    Ok(())
}

pub async fn run_stdio_server(polar: Option<crate::Polar>) -> anyhow::Result<()> {
    let (service, messages) = LspService::new(|client| Backend {
        client,
        analyzer: Arc::new(RwLock::new(polar.unwrap_or_default())),
        root: Default::default(),
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
