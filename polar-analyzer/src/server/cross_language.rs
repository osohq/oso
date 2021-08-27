use serde::{Deserialize, Serialize};
use tower_lsp::jsonrpc::Result;

use super::Backend;

impl Backend {
    pub async fn get_all_symbols(&self) -> Result<Symbols> {
        self.client.send_custom_request::<GetAllSymbols>(()).await
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]

struct GetAllSymbols;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Symbols {
    pub classes: Vec<String>,
}

impl lsp_types::request::Request for GetAllSymbols {
    type Params = ();

    type Result = Symbols;

    const METHOD: &'static str = "polar-analyzer/getAllSymbols";
}
