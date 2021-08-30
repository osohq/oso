use lsp_types::{SymbolInformation, SymbolKind};
use polar_core::terms::{Symbol, Term, Value};
use serde::{Deserialize, Serialize};
use tower_lsp::jsonrpc::Result;
use tracing::{debug, error};

use super::Backend;

impl Backend {
    async fn get_all_symbols(&self, names: Option<Vec<String>>) -> Result<Symbols> {
        self.client
            .send_custom_request::<GetAllSymbols>(GetAllSymbolsParams { names })
            .await
    }

    /// Get all classes present in the Polar files, and
    /// attempt to lookup corresponding symbols from the
    /// IDE.
    pub async fn refresh_workspace_symbols(&self) {
        let polar = self.get_analyzer().await;
        let classes = polar
            .get_files()
            .iter()
            .flat_map(|f| {
                polar
                    .get_term_info(f)
                    .into_iter()
                    .filter_map(|t| match t.r#type.as_str() {
                        "Variable" | "Pattern" => Some(t.name),
                        _ => None,
                    })
            })
            .collect();
        let symbols_res = self.get_all_symbols(Some(classes)).await;

        match symbols_res {
            Ok(symbols) => {
                debug!("Adding symbols to Polar: {:#?}", symbols);
                for symbol in symbols.symbols.into_iter() {
                    if symbol.kind == SymbolKind::Class {
                        polar.inner.register_constant(
                            Symbol(symbol.name),
                            Term::new_temporary(Value::Boolean(false)),
                        )
                    }
                }
            }
            Err(e) => {
                error!("Couldn't get symbols: {}", e)
            }
        }
    }
}

enum GetAllSymbols {}

#[derive(Deserialize, Serialize, Clone, Debug)]

struct GetAllSymbolsParams {
    /// Optional list of specific names to search for
    pub names: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Symbols {
    pub symbols: Vec<SymbolInformation>,
}

impl lsp_types::request::Request for GetAllSymbols {
    type Params = GetAllSymbolsParams;

    type Result = Symbols;

    const METHOD: &'static str = "polar-analyzer/getAllSymbols";
}
