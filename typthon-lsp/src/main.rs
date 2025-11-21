/*!
Typthon Language Server Protocol (LSP) Implementation

Provides editor integration for:
- Real-time type checking
- Code completion
- Go to definition
- Hover information
- Diagnostics
*/

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use dashmap::DashMap;
use std::sync::Arc;

mod analyzer;
mod diagnostics;
mod completion;

use analyzer::DocumentAnalyzer;
use diagnostics::DiagnosticCollector;

/// The Typthon Language Server
pub struct TypthonLanguageServer {
    client: Client,
    documents: Arc<DashMap<String, String>>,
    analyzer: Arc<DocumentAnalyzer>,
}

impl TypthonLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
            analyzer: Arc::new(DocumentAnalyzer::new()),
        }
    }

    async fn analyze_document(&self, uri: &str) {
        if let Some(content) = self.documents.get(uri) {
            let diagnostics = self.analyzer.analyze(content.value());

            let lsp_diagnostics: Vec<Diagnostic> = diagnostics
                .into_iter()
                .map(|d| Diagnostic {
                    range: Range {
                        start: Position {
                            line: d.line as u32,
                            character: d.col as u32,
                        },
                        end: Position {
                            line: d.line as u32,
                            character: (d.col + 10) as u32, // Approximate end
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    source: Some("typthon".to_string()),
                    message: d.message,
                    related_information: None,
                    tags: None,
                    code_description: None,
                    data: None,
                })
                .collect();

            self.client
                .publish_diagnostics(uri.parse().unwrap(), lsp_diagnostics, None)
                .await;
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for TypthonLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        tracing::info!("Typthon LSP server initializing");

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("typthon".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "typthon-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("Typthon LSP server initialized");
        self.client
            .log_message(MessageType::INFO, "Typthon LSP server started")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("Typthon LSP server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;

        tracing::info!("Document opened: {}", uri);
        self.documents.insert(uri.clone(), text);
        self.analyze_document(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();

        if let Some(change) = params.content_changes.first() {
            tracing::debug!("Document changed: {}", uri);
            self.documents.insert(uri.clone(), change.text.clone());
            self.analyze_document(&uri).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        tracing::info!("Document saved: {}", uri);
        self.analyze_document(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        tracing::info!("Document closed: {}", uri);
        self.documents.remove(&uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        tracing::debug!("Hover request at {}:{}", position.line, position.character);

        if let Some(content) = self.documents.get(&uri.to_string()) {
            let info = self.analyzer.get_hover_info(
                content.value(),
                position.line as usize,
                position.character as usize,
            );

            if let Some(text) = info {
                return Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(text)),
                    range: None,
                }));
            }
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        tracing::debug!("Completion request at {}:{}", position.line, position.character);

        if let Some(content) = self.documents.get(&uri.to_string()) {
            let completions = self.analyzer.get_completions(
                content.value(),
                position.line as usize,
                position.character as usize,
            );

            let items: Vec<CompletionItem> = completions
                .into_iter()
                .map(|c| CompletionItem {
                    label: c.label,
                    kind: Some(c.kind),
                    detail: Some(c.detail),
                    documentation: c.documentation.map(|d| {
                        Documentation::String(d)
                    }),
                    ..Default::default()
                })
                .collect();

            return Ok(Some(CompletionResponse::Array(items)));
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        tracing::debug!("Go to definition at {}:{}", position.line, position.character);

        if let Some(content) = self.documents.get(&uri.to_string()) {
            if let Some(location) = self.analyzer.get_definition(
                content.value(),
                position.line as usize,
                position.character as usize,
            ) {
                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position {
                            line: location.line as u32,
                            character: location.col as u32,
                        },
                        end: Position {
                            line: location.line as u32,
                            character: (location.col + location.length) as u32,
                        },
                    },
                })));
            }
        }

        Ok(None)
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Typthon Language Server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| TypthonLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}

