use serde::{Deserialize, Serialize};
use super::ProjectType;

/// Information about a registered parser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserInfo {
    /// Unique parser ID
    pub id: String,

    /// Display name
    pub display_name: String,

    /// Description
    pub description: String,

    /// Version
    pub version: String,

    /// Supported file extensions
    pub file_extensions: Vec<String>,

    /// Marker files for detection
    pub marker_files: Vec<String>,

    /// Marker directories for detection
    pub marker_dirs: Vec<String>,

    /// Project type this parser handles
    pub project_type: ProjectType,

    /// Primary color (hex)
    pub primary_color: String,

    /// Is currently available
    pub is_available: bool,
}

/// Parser registry - stores information about available parsers
pub struct ParserRegistry {
    parsers: Vec<ParserInfo>,
}

impl ParserRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
        }
    }

    /// Create registry with default parsers
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Register Delphi parser
        registry.register(ParserInfo {
            id: "delphi".to_string(),
            display_name: "Delphi / Object Pascal".to_string(),
            description: "Parser for Delphi and Lazarus projects".to_string(),
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
        });

        // Register Laravel parser
        registry.register(ParserInfo {
            id: "laravel".to_string(),
            display_name: "Laravel (PHP)".to_string(),
            description: "Parser for Laravel PHP framework projects".to_string(),
            version: "1.0.0".to_string(),
            file_extensions: vec!["php".to_string()],
            marker_files: vec!["composer.json".to_string(), "artisan".to_string()],
            marker_dirs: vec![
                "app/Http/Controllers".to_string(),
                "resources/views".to_string(),
            ],
            project_type: ProjectType::Laravel,
            primary_color: "#FF2D20".to_string(),
            is_available: true,
        });

        // Register Node.js parser (placeholder)
        registry.register(ParserInfo {
            id: "nodejs".to_string(),
            display_name: "Node.js / TypeScript".to_string(),
            description: "Parser for Node.js and TypeScript projects".to_string(),
            version: "0.1.0".to_string(),
            file_extensions: vec![
                "js".to_string(),
                "ts".to_string(),
                "jsx".to_string(),
                "tsx".to_string(),
            ],
            marker_files: vec!["package.json".to_string(), "tsconfig.json".to_string()],
            marker_dirs: vec!["node_modules".to_string()],
            project_type: ProjectType::NodeJs,
            primary_color: "#339933".to_string(),
            is_available: false, // Not yet implemented
        });

        registry
    }

    /// Register a new parser
    pub fn register(&mut self, info: ParserInfo) {
        self.parsers.push(info);
    }

    /// Get parser by ID
    pub fn get(&self, id: &str) -> Option<&ParserInfo> {
        self.parsers.iter().find(|p| p.id == id)
    }

    /// Get parser for a project type
    pub fn get_for_type(&self, project_type: &ProjectType) -> Option<&ParserInfo> {
        self.parsers.iter().find(|p| p.project_type == *project_type)
    }

    /// List all registered parsers
    pub fn list(&self) -> &[ParserInfo] {
        &self.parsers
    }

    /// List only available parsers
    pub fn list_available(&self) -> Vec<&ParserInfo> {
        self.parsers.iter().filter(|p| p.is_available).collect()
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// Global registry instance
lazy_static::lazy_static! {
    pub static ref PARSER_REGISTRY: ParserRegistry = ParserRegistry::with_defaults();
}
