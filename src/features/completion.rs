use tower_lsp::lsp_types::{CompletionItem, CompletionParams, CompletionResponse};

#[derive(Default)]
pub struct CompletionModule {}

impl CompletionModule {
    pub fn get_response(&self, _: CompletionParams) -> Option<CompletionResponse> {
        let mut items: Vec<CompletionItem> = Vec::new();

        items.push(CompletionItem::new_simple(
            "Test completion item".to_string(),
            "Details".to_string(),
        ));

        let response = CompletionResponse::Array(items);

        Some(response)
    }
}
