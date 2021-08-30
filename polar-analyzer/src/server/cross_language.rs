use serde::{Deserialize, Serialize};
use tower_lsp::jsonrpc::Result;

use super::Backend;

impl Backend {
    pub async fn get_all_symbols(&self, names: Option<Vec<String>>) -> Result<Symbols> {
        self.client
            .send_custom_request::<GetAllSymbols>(GetAllSymbolsParams { names })
            .await
    }
}

enum GetAllSymbols {}

#[derive(Deserialize, Serialize, Clone, Debug)]

struct GetAllSymbolsParams {
    pub names: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Symbols {
    pub classes: Vec<String>,
}

impl lsp_types::request::Request for GetAllSymbols {
    type Params = GetAllSymbolsParams;

    type Result = Symbols;

    const METHOD: &'static str = "polar-analyzer/getAllSymbols";
}
