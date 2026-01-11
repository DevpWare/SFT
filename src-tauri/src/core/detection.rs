use serde::{Deserialize, Serialize};
use std::path::Path;
use super::ProjectType;

/// Result of project type detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// Detected project type
    pub project_type: ProjectType,

    /// Detection confidence (0.0 - 1.0)
    pub confidence: f32,

    /// Recommended parser ID
    pub parser_id: String,

    /// Marker files found
    pub marker_files_found: Vec<String>,

    /// Is multi-language project
    pub is_multi_language: bool,

    /// Secondary types detected (for multi-language projects)
    pub secondary_types: Vec<(ProjectType, f32)>,
}

impl Default for DetectionResult {
    fn default() -> Self {
        Self {
            project_type: ProjectType::Unknown,
            confidence: 0.0,
            parser_id: String::new(),
            marker_files_found: Vec::new(),
            is_multi_language: false,
            secondary_types: Vec::new(),
        }
    }
}

/// Project type detector
pub struct ProjectDetector;

impl ProjectDetector {
    /// Detect project type from directory
    pub fn detect(root_path: &Path) -> DetectionResult {
        let mut scores: Vec<(ProjectType, f32, Vec<String>)> = Vec::new();

        // Check for Delphi
        let (delphi_score, delphi_markers) = Self::detect_delphi(root_path);
        if delphi_score > 0.0 {
            scores.push((ProjectType::Delphi, delphi_score, delphi_markers));
        }

        // Check for Laravel
        let (laravel_score, laravel_markers) = Self::detect_laravel(root_path);
        if laravel_score > 0.0 {
            scores.push((ProjectType::Laravel, laravel_score, laravel_markers));
        }

        // Check for Node.js
        let (nodejs_score, nodejs_markers) = Self::detect_nodejs(root_path);
        if nodejs_score > 0.0 {
            scores.push((ProjectType::NodeJs, nodejs_score, nodejs_markers));
        }

        // Check for generic PHP
        let (php_score, php_markers) = Self::detect_php(root_path);
        if php_score > 0.0 && laravel_score < 0.5 {
            scores.push((ProjectType::Php, php_score, php_markers));
        }

        // Sort by confidence descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if scores.is_empty() {
            return DetectionResult::default();
        }

        let (project_type, confidence, markers) = scores.remove(0);
        let parser_id = Self::get_parser_id(&project_type);

        // Check for multi-language
        let secondary_types: Vec<(ProjectType, f32)> = scores
            .iter()
            .filter(|(_, conf, _)| *conf > 0.3)
            .map(|(pt, conf, _)| (pt.clone(), *conf))
            .collect();

        DetectionResult {
            project_type,
            confidence,
            parser_id,
            marker_files_found: markers,
            is_multi_language: !secondary_types.is_empty(),
            secondary_types,
        }
    }

    fn detect_delphi(root_path: &Path) -> (f32, Vec<String>) {
        let mut score = 0.0f32;
        let mut markers = Vec::new();

        // Check for project files
        for ext in ["dpr", "dproj", "groupproj"] {
            if Self::has_files_with_extension(root_path, ext) {
                score += 0.4;
                markers.push(format!("*.{}", ext));
            }
        }

        // Check for .pas files
        if Self::has_files_with_extension(root_path, "pas") {
            score += 0.3;
            markers.push("*.pas".to_string());
        }

        // Check for form files
        if Self::has_files_with_extension(root_path, "dfm")
            || Self::has_files_with_extension(root_path, "fmx")
        {
            score += 0.2;
            markers.push("*.dfm/*.fmx".to_string());
        }

        (score.min(1.0), markers)
    }

    fn detect_laravel(root_path: &Path) -> (f32, Vec<String>) {
        let mut score = 0.0f32;
        let mut markers = Vec::new();

        // Check composer.json for laravel/framework
        let composer_path = root_path.join("composer.json");
        if composer_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&composer_path) {
                if content.contains("laravel/framework") {
                    score += 0.6;
                    markers.push("composer.json (laravel/framework)".to_string());
                }
            }
        }

        // Check for artisan
        if root_path.join("artisan").exists() {
            score += 0.2;
            markers.push("artisan".to_string());
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
                markers.push(dir.to_string());
            }
        }

        (score.min(1.0), markers)
    }

    fn detect_nodejs(root_path: &Path) -> (f32, Vec<String>) {
        let mut score = 0.0f32;
        let mut markers = Vec::new();

        // Check for package.json
        if root_path.join("package.json").exists() {
            score += 0.4;
            markers.push("package.json".to_string());
        }

        // Check for tsconfig.json
        if root_path.join("tsconfig.json").exists() {
            score += 0.2;
            markers.push("tsconfig.json".to_string());
        }

        // Check for TypeScript/JavaScript files
        if Self::has_files_with_extension(root_path, "ts")
            || Self::has_files_with_extension(root_path, "tsx")
        {
            score += 0.2;
            markers.push("*.ts/*.tsx".to_string());
        }

        if Self::has_files_with_extension(root_path, "js")
            || Self::has_files_with_extension(root_path, "jsx")
        {
            score += 0.1;
        }

        (score.min(1.0), markers)
    }

    fn detect_php(root_path: &Path) -> (f32, Vec<String>) {
        let mut score = 0.0f32;
        let mut markers = Vec::new();

        // Check for composer.json
        if root_path.join("composer.json").exists() {
            score += 0.3;
            markers.push("composer.json".to_string());
        }

        // Check for PHP files
        if Self::has_files_with_extension(root_path, "php") {
            score += 0.4;
            markers.push("*.php".to_string());
        }

        (score.min(1.0), markers)
    }

    fn has_files_with_extension(root_path: &Path, ext: &str) -> bool {
        if let Ok(entries) = std::fs::read_dir(root_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_ext) = path.extension() {
                        if file_ext.to_str().unwrap_or("").eq_ignore_ascii_case(ext) {
                            return true;
                        }
                    }
                } else if path.is_dir() {
                    // Check one level deep
                    if let Ok(subentries) = std::fs::read_dir(&path) {
                        for subentry in subentries.flatten() {
                            let subpath = subentry.path();
                            if subpath.is_file() {
                                if let Some(file_ext) = subpath.extension() {
                                    if file_ext.to_str().unwrap_or("").eq_ignore_ascii_case(ext) {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn get_parser_id(project_type: &ProjectType) -> String {
        match project_type {
            ProjectType::Delphi => "delphi".to_string(),
            ProjectType::Laravel => "laravel".to_string(),
            ProjectType::NodeJs => "nodejs".to_string(),
            ProjectType::Php => "php".to_string(),
            _ => "unknown".to_string(),
        }
    }
}
