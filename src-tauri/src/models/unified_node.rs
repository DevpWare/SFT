use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unified node type - language independent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum UnifiedNodeType {
    // === Files ===
    SourceFile,
    ConfigFile,
    FormFile,

    // === Code structures ===
    Module,
    Class,
    Interface,
    Trait,
    Enum,
    Struct,

    // === Functions ===
    Function,
    Method,
    Constructor,
    Destructor,

    // === UI Components ===
    Component,
    Form,
    Page,
    View,

    // === Web/API ===
    Route,
    Controller,
    Middleware,

    // === Data ===
    Model,
    Migration,
    Table,
    Query,

    // === Other ===
    Package,
    Variable,
    Constant,
    Custom(String),
}

impl Default for UnifiedNodeType {
    fn default() -> Self {
        UnifiedNodeType::SourceFile
    }
}

/// 3D Position for graph visualization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Position3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Parameter information for functions/methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: Option<String>,
    pub default_value: Option<String>,
    pub is_optional: bool,
}

/// Node status for annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Ok,
    Review,
    Deprecated,
    Critical,
    Todo,
}

/// Extended node metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeMetadata {
    /// File size in bytes
    pub size_bytes: Option<u64>,

    /// Visibility (public, private, protected)
    pub visibility: Option<String>,

    /// Is abstract
    pub is_abstract: Option<bool>,

    /// Is static
    pub is_static: Option<bool>,

    /// Return type (for functions)
    pub return_type: Option<String>,

    /// Parameters (for functions)
    pub parameters: Option<Vec<ParameterInfo>>,

    /// Parent class (inheritance)
    pub parent_class: Option<String>,

    /// Implemented interfaces
    pub implements: Option<Vec<String>>,

    /// Used traits (PHP)
    pub uses_traits: Option<Vec<String>>,

    /// Documentation/comments
    pub documentation: Option<String>,

    /// User notes
    pub notes: Option<String>,

    /// User tags
    pub tags: Option<Vec<String>>,

    /// Node status (for annotations)
    pub status: Option<NodeStatus>,

    /// Additional language-specific data
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Unified graph node - language independent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedNode {
    /// Unique ID (hash of path + name)
    pub id: String,

    /// Node type
    pub node_type: UnifiedNodeType,

    /// Short display name
    pub name: String,

    /// Fully qualified name (e.g., App\Http\Controllers\UserController)
    pub qualified_name: String,

    /// Label for graph display
    pub label: String,

    /// Visual size (1-12)
    pub size: u8,

    /// Source language/framework
    pub language: String,

    /// Source file path
    pub file_path: Option<String>,

    /// Start line in file
    pub line_start: Option<u32>,

    /// End line in file
    pub line_end: Option<u32>,

    /// 3D position (optional, for persistence)
    pub position: Option<Position3D>,

    /// Language-specific metadata
    #[serde(default)]
    pub metadata: NodeMetadata,
}

impl UnifiedNode {
    pub fn new(id: String, node_type: UnifiedNodeType, name: String) -> Self {
        Self {
            id,
            label: name.clone(),
            qualified_name: name.clone(),
            name,
            node_type,
            size: 4,
            language: String::new(),
            file_path: None,
            line_start: None,
            line_end: None,
            position: None,
            metadata: NodeMetadata::default(),
        }
    }

    pub fn with_file(mut self, path: String) -> Self {
        self.file_path = Some(path);
        self
    }

    pub fn with_language(mut self, lang: &str) -> Self {
        self.language = lang.to_string();
        self
    }

    pub fn with_size(mut self, size: u8) -> Self {
        self.size = size.clamp(1, 12);
        self
    }
}
