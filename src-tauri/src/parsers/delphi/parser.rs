use async_trait::async_trait;
use std::path::Path;

use crate::core::{ParserInfo, ProjectType};
use crate::models::{
    ParseResult, ParsedFile, SourceFile, UnifiedEdge, UnifiedEdgeType, UnifiedNode,
    UnifiedNodeType,
};
use crate::parsers::common::{generate_id, scan_directory};
use crate::parsers::{
    ParserCapabilities, ParserConfig, ParserResult, ParseProgress, ProgressCallback, ProjectParser,
};

use super::pas_parser::PasParser;
use super::dfm_parser::DfmParser;

/// Delphi/Object Pascal project parser
pub struct DelphiParser {
    pas_parser: PasParser,
    dfm_parser: DfmParser,
}

impl DelphiParser {
    pub fn new() -> Self {
        Self {
            pas_parser: PasParser::new(),
            dfm_parser: DfmParser::new(),
        }
    }

    fn classify_node_type(&self, file: &SourceFile) -> UnifiedNodeType {
        match file.extension.to_lowercase().as_str() {
            "pas" => UnifiedNodeType::Module,
            "dfm" | "fmx" => UnifiedNodeType::Form,
            "dpr" => UnifiedNodeType::SourceFile,
            "dpk" => UnifiedNodeType::Package,
            _ => UnifiedNodeType::SourceFile,
        }
    }
}

impl Default for DelphiParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProjectParser for DelphiParser {
    fn info(&self) -> ParserInfo {
        ParserInfo {
            id: "delphi".to_string(),
            display_name: "Delphi / Object Pascal".to_string(),
            description: "Parser for Delphi, Lazarus, and Object Pascal projects".to_string(),
            version: "1.0.0".to_string(),
            file_extensions: vec![
                "pas".to_string(),
                "dfm".to_string(),
                "fmx".to_string(),
                "dpr".to_string(),
                "dpk".to_string(),
            ],
            marker_files: vec!["*.dpr".to_string(), "*.dproj".to_string()],
            marker_dirs: vec![],
            project_type: ProjectType::Delphi,
            primary_color: "#E31D1D".to_string(),
            is_available: true,
        }
    }

    fn default_config(&self) -> ParserConfig {
        ParserConfig {
            include_extensions: vec![
                "pas".to_string(),
                "dfm".to_string(),
                "fmx".to_string(),
                "dpr".to_string(),
            ],
            exclude_dirs: vec![
                "__history".to_string(),
                "__recovery".to_string(),
                "Win32".to_string(),
                "Win64".to_string(),
                "Debug".to_string(),
                "Release".to_string(),
                ".git".to_string(),
            ],
            encoding: "utf-8".to_string(),
            parse_external_deps: false,
            max_depth: None,
            language_options: Default::default(),
        }
    }

    fn capabilities(&self) -> ParserCapabilities {
        ParserCapabilities {
            node_types: vec![
                "module".to_string(),
                "form".to_string(),
                "class".to_string(),
                "interface".to_string(),
                "function".to_string(),
                "procedure".to_string(),
            ],
            edge_types: vec![
                "uses".to_string(),
                "extends".to_string(),
                "implements".to_string(),
                "file_pair".to_string(),
            ],
            supports_incremental: false,
            supports_cancellation: true,
            available_metrics: vec![
                "lines_of_code".to_string(),
                "cyclomatic_complexity".to_string(),
            ],
        }
    }

    fn detect_confidence(&self, root_path: &Path) -> f32 {
        let mut score = 0.0f32;

        // Check for project files
        for ext in ["dpr", "dproj", "groupproj"] {
            if has_files_with_extension(root_path, ext) {
                score += 0.4;
            }
        }

        // Check for .pas files
        if has_files_with_extension(root_path, "pas") {
            score += 0.3;
        }

        // Check for form files
        if has_files_with_extension(root_path, "dfm")
            || has_files_with_extension(root_path, "fmx")
        {
            score += 0.2;
        }

        score.min(1.0)
    }

    fn can_handle_file(&self, file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext.to_lowercase().as_str(),
                "pas" | "dfm" | "fmx" | "dpr" | "dpk"
            )
        } else {
            false
        }
    }

    async fn scan_files(
        &self,
        root_path: &Path,
        config: &ParserConfig,
        _progress: Option<ProgressCallback>,
    ) -> ParserResult<Vec<SourceFile>> {
        let extensions: Vec<&str> = vec!["pas", "dfm", "fmx", "dpr", "dpk"];
        let exclude_dirs: Vec<&str> = config.exclude_dirs.iter().map(|s| s.as_str()).collect();

        Ok(scan_directory(root_path, &extensions, &exclude_dirs))
    }

    async fn parse_file(
        &self,
        file: &SourceFile,
        config: &ParserConfig,
    ) -> ParserResult<ParsedFile> {
        match file.extension.to_lowercase().as_str() {
            "pas" | "dpr" | "dpk" => self.pas_parser.parse(file, config).await,
            "dfm" | "fmx" => self.dfm_parser.parse(file, config).await,
            _ => Ok(ParsedFile::new(file.clone())),
        }
    }

    fn generate_nodes(&self, parse_result: &ParseResult) -> Vec<UnifiedNode> {
        let mut nodes = Vec::new();

        for parsed_file in &parse_result.files {
            // Create node for the file
            let file_id = generate_id(&parsed_file.source.path);
            let node_type = self.classify_node_type(&parsed_file.source);

            let mut node =
                UnifiedNode::new(file_id, node_type, parsed_file.source.name.clone())
                    .with_file(parsed_file.source.path.clone())
                    .with_language("delphi");

            // Set size based on file size
            let size = match parsed_file.source.size_bytes {
                0..=1000 => 2,
                1001..=5000 => 4,
                5001..=20000 => 6,
                20001..=50000 => 8,
                _ => 10,
            };
            node = node.with_size(size);

            nodes.push(node);

            // Create nodes for classes found in the file
            for symbol in &parsed_file.symbols {
                if matches!(
                    symbol.symbol_type,
                    crate::models::SymbolType::Class | crate::models::SymbolType::Interface
                ) {
                    let class_id = generate_id(&format!(
                        "{}::{}",
                        parsed_file.source.path, symbol.name
                    ));
                    let class_node = UnifiedNode::new(
                        class_id,
                        UnifiedNodeType::Class,
                        symbol.name.clone(),
                    )
                    .with_file(parsed_file.source.path.clone())
                    .with_language("delphi")
                    .with_size(4);

                    nodes.push(class_node);
                }
            }
        }

        nodes
    }

    fn generate_edges(
        &self,
        parse_result: &ParseResult,
        _nodes: &[UnifiedNode],
    ) -> Vec<UnifiedEdge> {
        let mut edges = Vec::new();

        for parsed_file in &parse_result.files {
            let source_id = generate_id(&parsed_file.source.path);

            // Create edges for dependencies (uses clauses)
            for dep in &parsed_file.dependencies {
                // Try to find the target file
                let target_id = generate_id(&dep.target);

                edges.push(
                    UnifiedEdge::new(source_id.clone(), target_id, UnifiedEdgeType::Uses)
                        .with_label(&dep.target),
                );
            }
        }

        // Detect file pairs (.pas <-> .dfm)
        let pairs = self.detect_file_pairs(
            &parse_result.files.iter().map(|f| f.source.clone()).collect::<Vec<_>>(),
        );
        for (pas_path, dfm_path) in pairs {
            let pas_id = generate_id(&pas_path);
            let dfm_id = generate_id(&dfm_path);
            edges.push(UnifiedEdge::new(pas_id, dfm_id, UnifiedEdgeType::FilePair));
        }

        edges
    }

    fn detect_file_pairs(&self, files: &[SourceFile]) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        // Group files by base name
        let pas_files: Vec<_> = files
            .iter()
            .filter(|f| f.extension.eq_ignore_ascii_case("pas"))
            .collect();

        let form_files: Vec<_> = files
            .iter()
            .filter(|f| {
                f.extension.eq_ignore_ascii_case("dfm") || f.extension.eq_ignore_ascii_case("fmx")
            })
            .collect();

        for pas in &pas_files {
            let pas_stem = std::path::Path::new(&pas.name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            for form in &form_files {
                let form_stem = std::path::Path::new(&form.name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                if pas_stem.eq_ignore_ascii_case(form_stem) {
                    pairs.push((pas.path.clone(), form.path.clone()));
                }
            }
        }

        pairs
    }
}

fn has_files_with_extension(root_path: &Path, ext: &str) -> bool {
    if let Ok(entries) = std::fs::read_dir(root_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_ext) = path.extension().and_then(|e| e.to_str()) {
                    if file_ext.eq_ignore_ascii_case(ext) {
                        return true;
                    }
                }
            }
        }
    }
    false
}
