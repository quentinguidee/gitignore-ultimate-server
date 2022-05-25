use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::features::completion::CompletionModule;
use crate::file::File;

pub struct Backend {
    client: Client,

    files: DashMap<String, File>,

    completion_module: CompletionModule,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Backend {
            client,

            files: DashMap::new(),

            completion_module: CompletionModule::default(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let completion_options = CompletionOptions {
            ..Default::default()
        };

        let workspace = WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            ..Default::default()
        };

        let capabilities = ServerCapabilities {
            completion_provider: Some(completion_options),
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            workspace: Some(workspace),
            ..Default::default()
        };

        Ok(InitializeResult {
            capabilities,
            ..InitializeResult::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Server initialized successfully.")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let url = params.text_document.uri;
        let text = params.text_document.text;

        self.files.insert(url.to_string(), File::new(url, text));
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut file = match self.files.get_mut(&params.text_document.uri.to_string()) {
            Some(file) => (file),
            None => {
                let error = format!(
                    "The file {url} is not opened on the server.",
                    url = params.text_document.uri.to_string()
                );
                self.client.log_message(MessageType::ERROR, error).await;
                return;
            }
        };

        for change in params.content_changes {
            file.apply_change(change)
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.files.remove(&params.text_document.uri.to_string());
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(self.completion_module.get_response(params))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
