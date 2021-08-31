mod completion;
mod config;
mod cross_language;
mod documents;
mod hover;
mod symbols;

use lsp_types::*;
use tokio::sync::{Mutex, OwnedMutexGuard};
use tower_lsp::jsonrpc::Result;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;

use tracing::{debug, error, trace, warn};

pub struct Backend {
    pub client: Client,
    pub analyzer: Arc<Mutex<crate::Polar>>,
    pub root: Arc<Mutex<Option<String>>>,
}

impl std::fmt::Debug for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Backend")
            // .field("client", &self.client)
            // .field("analyzer", &self.analyzer)
            // .field("root", &self.root)
            .finish()
    }
}

impl Backend {
    pub async fn uri_to_string(&self, uri: &impl ToString) -> String {
        if let Some(root) = self.root.lock().await.as_ref() {
            uri.to_string().replace(root, "")
        } else {
            uri.to_string()
        }
    }
    async fn get_analyzer(&self) -> AnalyzerLock {
        trace!("acquiring analyzer lock");

        let res = tokio::time::timeout(
            Duration::from_millis(30_000),
            self.analyzer.clone().lock_owned(),
        )
        .await
        .expect("acquiring a lock on polar-analyzer timed out");
        AnalyzerLock(res)
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    #[tracing::instrument]
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(root) = params.root_uri {
            self.root.lock().await.replace(root.to_string() + "/");
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

    #[tracing::instrument]
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::Info, "server initialized!")
            .await;
    }

    #[tracing::instrument]
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument]
    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        let _ = params;
        warn!("Got a workspace/didChangeWorkspaceFolders notification, but it is not implemented");
    }

    #[tracing::instrument]
    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let _ = params;
        tracing::trace!("get workspace symbols");
        Ok(Some(vec![]))
    }

    #[tracing::instrument]
    async fn did_rename_files(&self, params: RenameFilesParams) {
        if let Err(e) = self.rename_files(params).await {
            error!("{}", e)
        }
    }

    #[tracing::instrument]
    async fn did_delete_files(&self, params: DeleteFilesParams) {
        if let Err(e) = self.delete_files(params).await {
            error!("{}", e)
        }
    }

    #[tracing::instrument]
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.refresh_workspace_symbols().await;
        let uri = params.text_document.uri.clone();
        if let Err(e) = self.open_document(params).await {
            error!("{}", e)
        }
        self.refresh_workspace_symbols().await;
        self.revalidate_document(uri).await;
    }

    #[tracing::instrument]
    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.refresh_workspace_symbols().await;
        self.revalidate_document(params.text_document.uri).await;
    }

    #[tracing::instrument]
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Err(e) = self.edit_document(params).await {
            error!("{}", e)
        }
    }

    #[tracing::instrument]
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(completion::get_completions(params))
    }

    #[tracing::instrument]
    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        Ok(completion::resolve_completion(params))
    }

    #[tracing::instrument]
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let res = self.get_hover(params).await;
        trace!("Hover result: {:#?}", res);
        Ok(res)
    }

    #[tracing::instrument]
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        Ok(self.get_document_symbols(params).await)
    }
}

pub async fn run_tcp_server(polar: Option<crate::Polar>, port: u32) -> anyhow::Result<()> {
    let analyzer = Arc::new(Mutex::new(polar.unwrap_or_default()));
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
        analyzer: Arc::new(Mutex::new(polar.unwrap_or_default())),
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

struct AnalyzerLock(pub OwnedMutexGuard<crate::Polar>);

impl Deref for AnalyzerLock {
    type Target = crate::Polar;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AnalyzerLock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for AnalyzerLock {
    fn drop(&mut self) {
        trace!("releasing analyzer lock");
    }
}
