use std::path::Path;

use crate::core::{DetectionResult, ParserInfo, ParserRegistry, ProjectDetector, PARSER_REGISTRY};
use crate::models::SourceFile;
use crate::parsers::delphi::DelphiParser;
use crate::parsers::laravel::LaravelParser;
use crate::parsers::ProjectParser;

/// Detect project type from a directory path
#[tauri::command]
pub async fn detect_project_type(path: String) -> Result<DetectionResult, String> {
    let path = Path::new(&path);

    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    if !path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    Ok(ProjectDetector::detect(path))
}

/// List all available parsers
#[tauri::command]
pub fn list_parsers() -> Vec<ParserInfo> {
    PARSER_REGISTRY.list().to_vec()
}

/// Scan directory for source files
#[tauri::command]
pub async fn scan_directory(
    path: String,
    parser_id: Option<String>,
) -> Result<Vec<SourceFile>, String> {
    let root_path = Path::new(&path);

    if !root_path.exists() {
        return Err("Path does not exist".to_string());
    }

    // Detect or use specified parser
    let parser_id = parser_id.unwrap_or_else(|| {
        let detection = ProjectDetector::detect(root_path);
        detection.parser_id
    });

    // Get appropriate parser and scan
    let files = match parser_id.as_str() {
        "delphi" => {
            let parser = DelphiParser::new();
            let config = parser.default_config();
            parser
                .scan_files(root_path, &config, None)
                .await
                .map_err(|e| e.to_string())?
        }
        "laravel" => {
            let parser = LaravelParser::new();
            let config = parser.default_config();
            parser
                .scan_files(root_path, &config, None)
                .await
                .map_err(|e| e.to_string())?
        }
        _ => return Err(format!("Unknown parser: {}", parser_id)),
    };

    Ok(files)
}
