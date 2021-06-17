use super::bindings::Bindings;
use super::error::{PolarResult, ValidationError};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ResultEvent {
    bindings: Bindings,
}

pub fn validate_roles_config(validation_query_results: &str) -> PolarResult<()> {
    eprintln!("{}", validation_query_results);
    let roles_config: Vec<Vec<ResultEvent>> = serde_json::from_str(&validation_query_results)
        .map_err(|_| ValidationError("Invalid config query result".to_string()))?;
    Ok(())
}
