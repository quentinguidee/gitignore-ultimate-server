use std::{
    fs::{read_dir, DirEntry, ReadDir},
    io::Error,
    path::Path,
};

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
};

use crate::workspace::Workspace;

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
        let gitignore_document_position = params.text_document_position;
        let line = gitignore_document_position.position.line;

        // Get gitignore file
        let gitignore_file_uri = gitignore_document_position.text_document.uri;
        let gitignore_file = match workspace.files.get(&gitignore_file_uri.to_string()) {
            Some(file) => file,
            None => {
                return Err(format!(
                    "The file {url} is not opened on the server.",
                    url = gitignore_file_uri.to_string()
                ));
            }
        };

        let line_content = gitignore_file.get_line_content(line);
        let line_content = line_content.trim();

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
        let paths = match read_dir(Path::join(&absolute_path, relative_path)) {
            Ok(paths) => paths,
            Err(error) => return Err(error.to_string()),
        };

        // Return items
        Ok(Self::completion_items_for_paths(paths, file_name))
    }
}
