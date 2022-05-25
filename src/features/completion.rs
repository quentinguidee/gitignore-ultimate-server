use std::{
    fs::{read_dir, DirEntry, ReadDir},
    io::Error,
    path::Path,
};

use tower_lsp::{
    lsp_types::{
        CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, MessageType,
    },
    Client,
};

use crate::workspace::Workspace;

#[derive(Default)]
pub struct CompletionModule {}

impl CompletionModule {
    fn completion_item_for_path(path: Result<DirEntry, Error>) -> Option<CompletionItem> {
        let path = match path {
            Ok(path) => path,
            Err(_) => return None,
        };

        let file_name = path.file_name();
        let file_name = match file_name.to_str() {
            Some(file_name) => file_name,
            None => return None,
        };

        let kind = if path.path().is_dir() {
            CompletionItemKind::FOLDER
        } else {
            CompletionItemKind::FILE
        };

        Some(CompletionItem {
            label: file_name.to_string(),
            detail: Some(path.path().display().to_string()),
            kind: Some(kind),
            ..Default::default()
        })
    }

    fn completion_items_for_paths(paths: ReadDir) -> CompletionResponse {
        let mut items: Vec<CompletionItem> = Vec::new();
        for path in paths {
            match Self::completion_item_for_path(path) {
                Some(item) => items.push(item),
                None => continue,
            }
        }
        CompletionResponse::Array(items)
    }

    pub async fn get_response(
        &self,
        params: CompletionParams,
        workspace: &Workspace,
        client: &Client,
    ) -> Option<CompletionResponse> {
        let gitignore_uri = &params.text_document_position.text_document.uri;

        let file = workspace.files.get(&gitignore_uri.to_string());
        let file = match file {
            Some(file) => (file),
            None => {
                let error = format!(
                    "The file {url} is not opened on the server.",
                    url = params.text_document_position.text_document.uri.to_string()
                );
                client.log_message(MessageType::ERROR, error).await;
                return None;
            }
        };
        let file = &file.value();

        let line_content = file.get_line_content(params.text_document_position.position.line);
        let line_content = line_content.trim();

        let gitignore_path = file.path();
        let gitignore_path = match Path::new(&gitignore_path).parent() {
            Some(path) => path,
            None => return None,
        };

        client.log_message(MessageType::INFO, line_content).await;

        let path = Path::new(line_content);
        let path = if !line_content.ends_with("/") {
            path.parent().unwrap_or_else(|| Path::new(""))
        } else {
            path
        };

        let complete_path = Path::join(gitignore_path, path);

        let paths = match read_dir(complete_path) {
            Ok(paths) => paths,
            Err(error) => {
                client.log_message(MessageType::ERROR, error).await;
                return None;
            }
        };

        Some(Self::completion_items_for_paths(paths))
    }
}
