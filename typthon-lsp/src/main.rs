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
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::NAMESPACE,
                                    SemanticTokenType::CLASS,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::PARAMETER,
                                    SemanticTokenType::PROPERTY,
                                    SemanticTokenType::METHOD,
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::TYPE,
                                ],
                                token_modifiers: vec![
                                    SemanticTokenModifier::DEFINITION,
                                    SemanticTokenModifier::READONLY,
                                ],
                            },
                            range: Some(true),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            work_done_progress_options: WorkDoneProgressOptions::default(),
                        }
                    )
                ),
                inlay_hint_provider: Some(OneOf::Left(true)),
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

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        tracing::debug!("Find references at {}:{}", position.line, position.character);

        if let Some(content) = self.documents.get(&uri.to_string()) {
            let references = self.analyzer.find_references(
                content.value(),
                position.line as usize,
                position.character as usize,
            );

            let locations: Vec<Location> = references
                .into_iter()
                .map(|r| Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position {
                            line: r.line as u32,
                            character: r.col as u32,
                        },
                        end: Position {
                            line: r.line as u32,
                            character: (r.col + r.length) as u32,
                        },
                    },
                })
                .collect();

            return Ok(Some(locations));
        }

        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        tracing::debug!("Rename at {}:{} to {}", position.line, position.character, new_name);

        if let Some(content) = self.documents.get(&uri.to_string()) {
            let references = self.analyzer.find_references(
                content.value(),
                position.line as usize,
                position.character as usize,
            );

            let edits: Vec<TextEdit> = references
                .into_iter()
                .map(|r| TextEdit {
                    range: Range {
                        start: Position {
                            line: r.line as u32,
                            character: r.col as u32,
                        },
                        end: Position {
                            line: r.line as u32,
                            character: (r.col + r.length) as u32,
                        },
                    },
                    new_text: new_name.clone(),
                })
                .collect();

            let mut changes = std::collections::HashMap::new();
            changes.insert(uri, edits);

            return Ok(Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }));
        }

        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let _range = params.range;

        tracing::debug!("Code action request");

        if let Some(_content) = self.documents.get(&uri.to_string()) {
            let mut actions = Vec::new();

            // Add import statement action
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Add missing import".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None,
                edit: None,
                command: None,
                is_preferred: Some(true),
                disabled: None,
                data: None,
            }));

            // Add type annotation action
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Add type annotation".to_string(),
                kind: Some(CodeActionKind::REFACTOR),
                diagnostics: None,
                edit: None,
                command: None,
                is_preferred: Some(false),
                disabled: None,
                data: None,
            }));

            return Ok(Some(actions));
        }

        Ok(None)
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        tracing::debug!("Signature help at {}:{}", position.line, position.character);

        if let Some(content) = self.documents.get(&uri.to_string()) {
            // Extract function name at cursor
            let lines: Vec<&str> = content.lines().collect();
            if position.line as usize >= lines.len() {
                return Ok(None);
            }

            let line = lines[position.line as usize];
            let before_cursor = &line[..position.character.min(line.len() as u32) as usize];

            // Simple pattern to find function call
            if let Some(open_paren) = before_cursor.rfind('(') {
                let func_start = before_cursor[..open_paren]
                    .rfind(|c: char| !c.is_alphanumeric() && c != '_')
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let func_name = &before_cursor[func_start..open_paren];

                // Provide signature for known functions
                let signature_info = match func_name {
                    "print" => Some(SignatureInformation {
                        label: "print(*args, sep=' ', end='\\n', file=None, flush=False)".to_string(),
                        documentation: Some(Documentation::String(
                            "Print values to a stream, or to sys.stdout by default.".to_string()
                        )),
                        parameters: Some(vec![
                            ParameterInformation {
                                label: ParameterLabel::Simple("*args".to_string()),
                                documentation: Some(Documentation::String("Values to print".to_string())),
                            },
                        ]),
                        active_parameter: None,
                    }),
                    "len" => Some(SignatureInformation {
                        label: "len(obj)".to_string(),
                        documentation: Some(Documentation::String(
                            "Return the length of an object.".to_string()
                        )),
                        parameters: Some(vec![
                            ParameterInformation {
                                label: ParameterLabel::Simple("obj".to_string()),
                                documentation: Some(Documentation::String("Object to measure".to_string())),
                            },
                        ]),
                        active_parameter: None,
                    }),
                    _ => None,
                };

                if let Some(sig) = signature_info {
                    return Ok(Some(SignatureHelp {
                        signatures: vec![sig],
                        active_signature: Some(0),
                        active_parameter: Some(0),
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;

        tracing::debug!("Semantic tokens request");

        if let Some(content) = self.documents.get(&uri.to_string()) {
            let symbols = self.analyzer.extract_symbols(content.value());

            let mut data = Vec::new();
            let mut prev_line = 0u32;
            let mut prev_char = 0u32;

            for symbol in symbols {
                let line = symbol.line as u32;
                let char = symbol.col as u32;
                let length = symbol.length as u32;

                let token_type = match symbol.kind {
                    analyzer::SymbolKind::Class => 1,      // CLASS
                    analyzer::SymbolKind::Function => 2,   // FUNCTION
                    analyzer::SymbolKind::Variable => 3,   // VARIABLE
                    analyzer::SymbolKind::Parameter => 4,  // PARAMETER
                    analyzer::SymbolKind::Property => 5,   // PROPERTY
                    analyzer::SymbolKind::Method => 6,     // METHOD
                };

                let delta_line = if line >= prev_line { line - prev_line } else { 0 };
                let delta_char = if delta_line == 0 && char >= prev_char {
                    char - prev_char
                } else {
                    char
                };

                data.push(SemanticToken {
                    delta_line,
                    delta_start: delta_char,
                    length,
                    token_type,
                    token_modifiers_bitset: 0,
                });

                prev_line = line;
                prev_char = char;
            }

            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data,
            })));
        }

        Ok(None)
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let uri = params.text_document.uri;

        tracing::debug!("Inlay hint request");

        if let Some(content) = self.documents.get(&uri.to_string()) {
            let mut hints = Vec::new();
            let symbols = self.analyzer.extract_symbols(content.value());

            // Add type hints for variables without annotations
            for symbol in symbols {
                if matches!(symbol.kind, analyzer::SymbolKind::Variable) {
                    hints.push(InlayHint {
                        position: Position {
                            line: symbol.line as u32,
                            character: (symbol.col + symbol.length) as u32,
                        },
                        label: InlayHintLabel::String(": Unknown".to_string()),
                        kind: Some(InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: Some(InlayHintTooltip::String(
                            "Type could not be inferred".to_string()
                        )),
                        padding_left: None,
                        padding_right: None,
                        data: None,
                    });
                }
            }

            return Ok(Some(hints));
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

