use async_trait::async_trait;
use std::path::Path;

use crate::core::{ParserInfo, ProjectType};
use crate::models::{ParseResult, ParsedFile, SourceFile, UnifiedEdge, UnifiedNode};
use crate::parsers::common::scan_directory;
use crate::parsers::{
    ParserCapabilities, ParserConfig, ParserResult, ProgressCallback, ProjectParser,
};

/// Laravel PHP framework parser (placeholder implementation)
pub struct LaravelParser;

impl LaravelParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LaravelParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProjectParser for LaravelParser {
    fn info(&self) -> ParserInfo {
        ParserInfo {
            id: "laravel".to_string(),
            display_name: "Laravel (PHP)".to_string(),
            description: "Parser for Laravel PHP framework projects".to_string(),
            version: "0.1.0".to_string(),
            file_extensions: vec!["php".to_string()],
            marker_files: vec!["composer.json".to_string(), "artisan".to_string()],
            marker_dirs: vec![
                "app/Http/Controllers".to_string(),
                "resources/views".to_string(),
            ],
            project_type: ProjectType::Laravel,
            primary_color: "#FF2D20".to_string(),
            is_available: true,
        }
    }

    fn default_config(&self) -> ParserConfig {
        ParserConfig {
            include_extensions: vec!["php".to_string()],
            exclude_dirs: vec![
                "vendor".to_string(),
                "node_modules".to_string(),
                "storage".to_string(),
                "bootstrap/cache".to_string(),
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
                "controller".to_string(),
                "model".to_string(),
                "view".to_string(),
                "route".to_string(),
                "migration".to_string(),
            ],
            edge_types: vec![
                "uses".to_string(),
                "extends".to_string(),
                "routes".to_string(),
                "renders".to_string(),
            ],
            supports_incremental: false,
            supports_cancellation: true,
            available_metrics: vec!["lines_of_code".to_string()],
        }
    }

    fn detect_confidence(&self, root_path: &Path) -> f32 {
        let mut score = 0.0f32;

        // Check for composer.json with laravel/framework
        if let Ok(content) = std::fs::read_to_string(root_path.join("composer.json")) {
            if content.contains("laravel/framework") {
                score += 0.6;
            }
        }

        // Check for artisan
        if root_path.join("artisan").exists() {
            score += 0.2;
        }

        // Check for Laravel directories
        let laravel_dirs = [
            "app/Http/Controllers",
            "resources/views",
            "routes",
            "database/migrations",
        ];
        for dir in laravel_dirs {
            if root_path.join(dir).is_dir() {
                score += 0.05;
            }
        }

        score.min(1.0)
    }

    fn can_handle_file(&self, file_path: &Path) -> bool {
        file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("php"))
            .unwrap_or(false)
    }

    async fn scan_files(
        &self,
        root_path: &Path,
        config: &ParserConfig,
        _progress: Option<ProgressCallback>,
    ) -> ParserResult<Vec<SourceFile>> {
        let extensions: Vec<&str> = vec!["php"];
        let exclude_dirs: Vec<&str> = config.exclude_dirs.iter().map(|s| s.as_str()).collect();

        Ok(scan_directory(root_path, &extensions, &exclude_dirs))
    }

    async fn parse_file(
        &self,
        file: &SourceFile,
        _config: &ParserConfig,
    ) -> ParserResult<ParsedFile> {
        // Placeholder: just create empty parsed file
        // TODO: Implement actual PHP parsing
        Ok(ParsedFile::new(file.clone()))
    }

    fn generate_nodes(&self, parse_result: &ParseResult) -> Vec<UnifiedNode> {
        // Placeholder implementation
        parse_result
            .files
            .iter()
            .map(|f| {
                crate::parsers::common::generate_id(&f.source.path);
                UnifiedNode::new(
                    crate::parsers::common::generate_id(&f.source.path),
                    crate::models::UnifiedNodeType::SourceFile,
                    f.source.name.clone(),
                )
                .with_file(f.source.path.clone())
                .with_language("php")
            })
            .collect()
    }

    fn generate_edges(
        &self,
        _parse_result: &ParseResult,
        _nodes: &[UnifiedNode],
    ) -> Vec<UnifiedEdge> {
        // Placeholder: no edges yet
        Vec::new()
    }

    fn detect_file_pairs(&self, _files: &[SourceFile]) -> Vec<(String, String)> {
        // Laravel doesn't have file pairs like Delphi
        Vec::new()
    }
}
