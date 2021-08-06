use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, Documentation,
    InsertTextFormat,
};

pub fn get_completions(_params: CompletionParams) -> Option<CompletionResponse> {
    let resp = CompletionResponse::Array(vec![CompletionItem {
        label: "allow".to_string(),
        kind: Some(CompletionItemKind::Function),
        data: Some(serde_json::to_value(1).unwrap()),
        ..Default::default()
    }]);
    Some(resp)
}

pub fn resolve_completion(mut item: CompletionItem) -> CompletionItem {
    if let Some(index) = item
        .data
        .clone()
        .and_then(|d| serde_json::from_value::<u64>(d).ok())
    {
        match index {
            1 => {
                item.detail = Some("allow rule".to_string());
                item.documentation = Some(Documentation::String(
                    "allow actor to perform action on resource".to_string(),
                ));
                item.insert_text =
                    Some("allow(${1:actor}, ${2:action}, ${3:resource}) if\n    $0;".to_string());
                item.insert_text_format = Some(InsertTextFormat::Snippet);
            }
            d => eprintln!("Unsupported completion index: {}", d),
        }
    } else {
        eprintln!("No completion data to resolve for: {:#?}", item)
    }
    item
}
