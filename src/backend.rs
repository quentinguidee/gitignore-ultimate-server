use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::features::completion::CompletionModule;
use crate::workspace::Workspace;

pub struct Backend {
    client: Client,

    workspace: Workspace,

    completion_module: CompletionModule,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Backend {
            client,

            workspace: Workspace::new(),

            completion_module: CompletionModule::default(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let completion_options = CompletionOptions {
            trigger_characters: Some(vec![String::from("/")]),
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

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.workspace.open(params);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.workspace.apply_changes(params, &self.client).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.workspace.close(params);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        match self
            .completion_module
            .get_completion(params, &self.workspace)
        {
            Ok(completions) => Ok(Some(completions)),
            Err(error) => {
                self.client.log_message(MessageType::ERROR, error).await;
                Ok(None)
            }
        }
    }
}
