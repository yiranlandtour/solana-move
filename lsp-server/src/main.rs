use dashmap::DashMap;
use ropey::Rope;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<DashMap<Url, Rope>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("ccdsl".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::NAMESPACE,
                                    SemanticTokenType::TYPE,
                                    SemanticTokenType::CLASS,
                                    SemanticTokenType::ENUM,
                                    SemanticTokenType::INTERFACE,
                                    SemanticTokenType::STRUCT,
                                    SemanticTokenType::TYPE_PARAMETER,
                                    SemanticTokenType::PARAMETER,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::PROPERTY,
                                    SemanticTokenType::ENUM_MEMBER,
                                    SemanticTokenType::EVENT,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::METHOD,
                                    SemanticTokenType::MACRO,
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::MODIFIER,
                                    SemanticTokenType::COMMENT,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::REGEXP,
                                    SemanticTokenType::OPERATOR,
                                ],
                                token_modifiers: vec![
                                    SemanticTokenModifier::DECLARATION,
                                    SemanticTokenModifier::DEFINITION,
                                    SemanticTokenModifier::READONLY,
                                    SemanticTokenModifier::STATIC,
                                    SemanticTokenModifier::DEPRECATED,
                                    SemanticTokenModifier::ABSTRACT,
                                    SemanticTokenModifier::ASYNC,
                                    SemanticTokenModifier::MODIFICATION,
                                    SemanticTokenModifier::DOCUMENTATION,
                                    SemanticTokenModifier::DEFAULT_LIBRARY,
                                ],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(false),
                            ..Default::default()
                        },
                    ),
                ),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "CrossChain DSL Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        
        self.documents.insert(uri.clone(), Rope::from_str(&text));
        self.validate_document(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(changes) = params.content_changes.first() {
            self.documents.insert(uri.clone(), Rope::from_str(&changes.text));
            self.validate_document(uri).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.validate_document(params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        
        let completions = self.get_completions(&uri, position).await;
        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let hover_info = self.get_hover_info(&uri, position).await;
        Ok(hover_info)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let definition = self.find_definition(&uri, position).await;
        Ok(definition)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let edits = self.format_document(&uri).await;
        Ok(Some(edits))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let range = params.range;
        
        let actions = self.get_code_actions(&uri, range).await;
        Ok(Some(actions))
    }
}

impl Backend {
    async fn validate_document(&self, uri: Url) {
        if let Some(rope) = self.documents.get(&uri) {
            let text = rope.to_string();
            
            // Parse and analyze the document
            match cross_chain_dsl::Contract::parse(&text) {
                Ok(contract) => {
                    // Semantic analysis
                    let mut analyzer = cross_chain_dsl::semantic::SemanticAnalyzer::new();
                    if let Err(e) = analyzer.analyze(&contract) {
                        // Send error diagnostics
                        let diagnostic = Diagnostic {
                            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!("Semantic error: {}", e),
                            ..Default::default()
                        };
                        
                        self.client
                            .publish_diagnostics(uri.clone(), vec![diagnostic], None)
                            .await;
                    } else {
                        // Clear diagnostics if successful
                        self.client
                            .publish_diagnostics(uri.clone(), vec![], None)
                            .await;
                    }
                }
                Err(e) => {
                    // Send parse error diagnostics
                    let diagnostic = Diagnostic {
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: format!("Parse error: {}", e),
                        ..Default::default()
                    };
                    
                    self.client
                        .publish_diagnostics(uri.clone(), vec![diagnostic], None)
                        .await;
                }
            }
        }
    }

    async fn get_completions(&self, uri: &Url, position: Position) -> Vec<CompletionItem> {
        let mut completions = vec![];
        
        // Keywords
        for keyword in &["contract", "state", "public", "private", "fn", "let", "if", "else", "require", "emit", "return"] {
            completions.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("Keyword: {}", keyword)),
                ..Default::default()
            });
        }
        
        // Types
        for ty in &["u8", "u64", "u128", "bool", "address", "string", "map", "vec"] {
            completions.push(CompletionItem {
                label: ty.to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some(format!("Type: {}", ty)),
                ..Default::default()
            });
        }
        
        // Built-in functions
        completions.push(CompletionItem {
            label: "msg_sender".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Returns the address of the message sender".to_string()),
            insert_text: Some("msg_sender()".to_string()),
            ..Default::default()
        });
        
        // Contract template
        completions.push(CompletionItem {
            label: "contract".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Create a new contract".to_string()),
            insert_text: Some(
                "contract ${1:ContractName} {\n\tstate {\n\t\t$2\n\t}\n\t\n\tpublic fn ${3:initialize}($4) {\n\t\t$0\n\t}\n}".to_string()
            ),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });
        
        completions
    }

    async fn get_hover_info(&self, uri: &Url, position: Position) -> Option<Hover> {
        // Get the word at the position
        if let Some(rope) = self.documents.get(uri) {
            let line_idx = position.line as usize;
            let char_idx = position.character as usize;
            
            if line_idx < rope.len_lines() {
                let line = rope.line(line_idx).to_string();
                
                // Simple word extraction (can be improved)
                let word = extract_word_at_position(&line, char_idx);
                
                // Provide hover info based on the word
                let hover_text = match word.as_str() {
                    "contract" => "Defines a new smart contract",
                    "state" => "Declares state variables that persist on the blockchain",
                    "public" => "Makes a function callable from outside the contract",
                    "private" => "Restricts function access to within the contract",
                    "require" => "Asserts a condition and reverts if false",
                    "emit" => "Emits an event for off-chain monitoring",
                    "u64" => "64-bit unsigned integer",
                    "address" => "Blockchain address type",
                    "map" => "Key-value mapping data structure",
                    _ => return None,
                };
                
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("**{}**\n\n{}", word, hover_text),
                    }),
                    range: None,
                });
            }
        }
        
        None
    }

    async fn find_definition(&self, uri: &Url, position: Position) -> Option<GotoDefinitionResponse> {
        // This would require maintaining a symbol table
        // For now, return None
        None
    }

    async fn format_document(&self, uri: &Url) -> Vec<TextEdit> {
        if let Some(rope) = self.documents.get(uri) {
            let text = rope.to_string();
            
            // Simple formatting: ensure consistent indentation
            let formatted = format_ccdsl(&text);
            
            if formatted != text {
                return vec![TextEdit {
                    range: Range::new(
                        Position::new(0, 0),
                        Position::new(rope.len_lines() as u32, 0),
                    ),
                    new_text: formatted,
                }];
            }
        }
        
        vec![]
    }

    async fn get_code_actions(&self, uri: &Url, range: Range) -> CodeActionResponse {
        let mut actions = vec![];
        
        // Quick fix: Add missing semicolon
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Add missing semicolon".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            edit: Some(WorkspaceEdit {
                changes: Some([(
                    uri.clone(),
                    vec![TextEdit {
                        range,
                        new_text: ";".to_string(),
                    }],
                )].into_iter().collect()),
                ..Default::default()
            }),
            ..Default::default()
        }));
        
        // Refactor: Extract to function
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Extract to function".to_string(),
            kind: Some(CodeActionKind::REFACTOR_EXTRACT),
            ..Default::default()
        }));
        
        actions
    }
}

fn extract_word_at_position(line: &str, position: usize) -> String {
    let chars: Vec<char> = line.chars().collect();
    
    if position >= chars.len() {
        return String::new();
    }
    
    let mut start = position;
    let mut end = position;
    
    // Find word boundaries
    while start > 0 && chars[start - 1].is_alphanumeric() || chars[start - 1] == '_' {
        start -= 1;
    }
    
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }
    
    chars[start..end].iter().collect()
}

fn format_ccdsl(text: &str) -> String {
    // Simple formatter that ensures consistent indentation
    let mut formatted = String::new();
    let mut indent_level = 0;
    
    for line in text.lines() {
        let trimmed = line.trim();
        
        if trimmed.ends_with('}') && !trimmed.starts_with('{') {
            indent_level = indent_level.saturating_sub(1);
        }
        
        formatted.push_str(&"\t".repeat(indent_level));
        formatted.push_str(trimmed);
        formatted.push('\n');
        
        if trimmed.ends_with('{') {
            indent_level += 1;
        }
    }
    
    formatted
}

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Arc::new(DashMap::new()),
    });
    
    Server::new(stdin, stdout, socket).serve(service).await;
}