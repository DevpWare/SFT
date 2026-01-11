use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

use crate::core::{ParserInfo, ProjectType};
use crate::models::{ParseResult, ParsedFile, SourceFile, UnifiedEdge, UnifiedGraph, UnifiedNode};

/// Parser error types
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error in {file}: {message}")]
    Parse { file: String, message: String },

    #[error("Unsupported file type: {0}")]
    UnsupportedFile(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Cancelled")]
    Cancelled,
}

pub type ParserResult<T> = Result<T, ParseError>;

/// Parser configuration
#[derive(Debug, Clone, Default)]
pub struct ParserConfig {
    /// File extensions to include (empty = all parser-supported)
    pub include_extensions: Vec<String>,

    /// Directories to exclude from scanning
    pub exclude_dirs: Vec<String>,

    /// File encoding (default: utf-8)
    pub encoding: String,

    /// Whether to parse external dependencies
    pub parse_external_deps: bool,

    /// Maximum analysis depth
    pub max_depth: Option<u32>,

    /// Language-specific options
    pub language_options: HashMap<String, serde_json::Value>,
}

impl ParserConfig {
    pub fn new() -> Self {
        Self {
            encoding: "utf-8".to_string(),
            ..Default::default()
        }
    }

    pub fn with_exclude_dirs(mut self, dirs: Vec<String>) -> Self {
        self.exclude_dirs = dirs;
        self
    }
}

/// Parse progress information
#[derive(Debug, Clone)]
pub struct ParseProgress {
    pub phase: String,
    pub current: usize,
    pub total: usize,
    pub current_file: Option<String>,
    pub message: String,
}

/// Progress callback type
pub type ProgressCallback = Box<dyn Fn(ParseProgress) + Send + Sync>;

/// Parser capabilities (for UI)
#[derive(Debug, Clone)]
pub struct ParserCapabilities {
    /// Node types this parser can generate
    pub node_types: Vec<String>,

    /// Edge types this parser can generate
    pub edge_types: Vec<String>,

    /// Supports incremental parsing
    pub supports_incremental: bool,

    /// Supports cancellation
    pub supports_cancellation: bool,

    /// Available metrics
    pub available_metrics: Vec<String>,
}

/// Main trait for project parsers (Strategy Pattern)
///
/// Each implementation handles a specific project type/language.
/// The design allows:
/// - Automatic project type detection
/// - Parsing of individual files or complete projects
/// - Generation of unified nodes and edges
/// - Real-time progress with cancellation support
#[async_trait]
pub trait ProjectParser: Send + Sync {
    // ============================================
    // PARSER METADATA
    // ============================================

    /// Returns parser information
    fn info(&self) -> ParserInfo;

    /// Returns default configuration
    fn default_config(&self) -> ParserConfig;

    /// Returns parser capabilities
    fn capabilities(&self) -> ParserCapabilities;

    // ============================================
    // PROJECT DETECTION
    // ============================================

    /// Calculate confidence (0.0 - 1.0) that the directory is this project type
    fn detect_confidence(&self, root_path: &Path) -> f32;

    /// Check if parser can handle a specific file
    fn can_handle_file(&self, file_path: &Path) -> bool;

    // ============================================
    // FILE SCANNING
    // ============================================

    /// Scan directory and return relevant files
    async fn scan_files(
        &self,
        root_path: &Path,
        config: &ParserConfig,
        progress: Option<ProgressCallback>,
    ) -> ParserResult<Vec<SourceFile>>;

    // ============================================
    // PARSING
    // ============================================

    /// Parse a single file and extract information
    async fn parse_file(&self, file: &SourceFile, config: &ParserConfig)
        -> ParserResult<ParsedFile>;

    /// Parse complete project
    /// Default implementation calls parse_file for each file
    async fn parse_project(
        &self,
        root_path: &Path,
        files: &[SourceFile],
        config: &ParserConfig,
        progress: Option<ProgressCallback>,
    ) -> ParserResult<ParseResult> {
        let mut result = ParseResult::new();
        let total = files.len();

        for (index, file) in files.iter().enumerate() {
            if let Some(ref callback) = progress {
                callback(ParseProgress {
                    phase: "parsing".to_string(),
                    current: index,
                    total,
                    current_file: Some(file.path.clone()),
                    message: format!("Parsing {}", file.name),
                });
            }

            match self.parse_file(file, config).await {
                Ok(parsed) => result.add_parsed_file(parsed),
                Err(e) => result.add_error(file.path.clone(), e.to_string()),
            }
        }

        Ok(result)
    }

    // ============================================
    // GRAPH CONSTRUCTION
    // ============================================

    /// Generate unified nodes from parse result
    fn generate_nodes(&self, parse_result: &ParseResult) -> Vec<UnifiedNode>;

    /// Generate unified edges from parse result
    fn generate_edges(&self, parse_result: &ParseResult, nodes: &[UnifiedNode]) -> Vec<UnifiedEdge>;

    /// Build complete graph from parse result
    fn build_graph(&self, parse_result: &ParseResult) -> UnifiedGraph {
        let nodes = self.generate_nodes(parse_result);
        let edges = self.generate_edges(parse_result, &nodes);

        UnifiedGraph {
            nodes,
            edges,
            metadata: Default::default(),
        }
    }

    // ============================================
    // FILE PAIR DETECTION
    // ============================================

    /// Detect related file pairs (e.g., .pas <-> .dfm)
    fn detect_file_pairs(&self, files: &[SourceFile]) -> Vec<(String, String)>;
}
