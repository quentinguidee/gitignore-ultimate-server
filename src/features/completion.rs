use std::{
    fs::{read_dir, DirEntry, ReadDir},
    io::Error,
    path::Path,
};

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
    TextDocumentPositionParams,
};
use ultimate_server_core::file_system::workspace::Workspace;

#[derive(Default)]
pub struct CompletionModule {}

impl CompletionModule {
    fn completion_item_for_path(
        path: Result<DirEntry, Error>,
        current_completed_file_name: String,
    ) -> Option<CompletionItem> {
        let path = match path {
            Ok(path) => path,
            Err(_) => return None,
        };

        let file_name = path.file_name();
        let file_name = match file_name.to_str() {
            Some(file_name) => file_name,
            None => return None,
        };

        if current_completed_file_name.starts_with(".") && !file_name.starts_with(".") {
            return None;
        }

        let kind = if path.path().is_dir() {
            CompletionItemKind::FOLDER
        } else {
            CompletionItemKind::FILE
        };

        let label = file_name.to_string();
        let insert_text = match current_completed_file_name.find(".") {
            Some(last_dot_index) => label[(last_dot_index + 1)..].to_string(),
            None => label.clone(),
        };

        Some(CompletionItem {
            label,
            detail: Some(path.path().display().to_string()),
            kind: Some(kind),
            insert_text: Some(insert_text),
            ..Default::default()
        })
    }

    fn completion_items_for_paths(
        paths: ReadDir,
        current_completed_file_name: String,
    ) -> CompletionResponse {
        let mut items: Vec<CompletionItem> = Vec::new();
        for path in paths {
            match Self::completion_item_for_path(path, current_completed_file_name.clone()) {
                Some(item) => items.push(item),
                None => continue,
            }
        }
        CompletionResponse::Array(items)
    }

    pub fn get_completion(
        &self,
        params: CompletionParams,
        workspace: &Workspace,
    ) -> Result<CompletionResponse, String> {
        let TextDocumentPositionParams {
            position,
            text_document,
        } = params.text_document_position;

        // Get gitignore file
        let gitignore_file = match workspace.files.get(&text_document.uri.to_string()) {
            Some(file) => file,
            None => {
                return Err(format!(
                    "The file {url} is not opened on the server.",
                    url = text_document.uri.to_string()
                ));
            }
        };

        let line_content = gitignore_file.get_line_content(position.line);
        let mut line_content = line_content.trim();
        while line_content.starts_with("/") || line_content.starts_with("!") {
            line_content = &line_content[1..]
        }

        // Paths
        let absolute_path = match gitignore_file.directory() {
            Ok(path) => path,
            Err(error) => return Err(error),
        };

        let relative_path = Path::new(line_content);
        let relative_path = if !line_content.ends_with("/") {
            relative_path.parent().unwrap_or(relative_path)
        } else {
            relative_path
        };

        // Filename
        let file_name = match line_content.rsplit_once("/") {
            Some((_, file_name)) => file_name,
            None => line_content,
        };
        let file_name = file_name.to_string();

        // Search for files
        let path = Path::join(&absolute_path, relative_path);

        #[cfg(windows)]
        let path = path.strip_prefix("/").unwrap_or(&path);

        let paths = match read_dir(path) {
            Ok(paths) => paths,
            Err(error) => return Err(error.to_string()),
        };

        // Return items
        Ok(Self::completion_items_for_paths(paths, file_name))
    }
}
