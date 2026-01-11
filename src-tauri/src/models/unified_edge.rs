use serde::{Deserialize, Serialize};

/// Unified edge type - language independent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum UnifiedEdgeType {
    // === Code dependencies ===
    Uses,       // Uses/imports (use, import, require)
    Extends,    // Inherits from
    Implements, // Implements interface
    Includes,   // Includes (PHP include, Delphi $I)

    // === Structural relations ===
    Contains, // Contains (file -> classes)
    Defines,  // Defines (module -> function)
    BelongsTo,

    // === Calls ===
    Calls,        // Calls function/method
    Instantiates, // Instantiates class

    // === Files ===
    FilePair, // File pair (.pas <-> .dfm)

    // === Web ===
    Routes,  // Route to controller
    Renders, // Renders view

    // === Data ===
    QueriesTable, // Queries table
    HasRelation,  // Model relation (hasMany, belongsTo)

    // === Other ===
    References,     // Generic reference
    Custom(String), // Custom type
}

impl Default for UnifiedEdgeType {
    fn default() -> Self {
        UnifiedEdgeType::Uses
    }
}

/// Edge metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EdgeMetadata {
    /// Line number where the relation occurs
    pub line_number: Option<u32>,

    /// Is conditional dependency
    pub is_conditional: Option<bool>,

    /// Is dev dependency (devDependencies)
    pub is_dev_dependency: Option<bool>,

    /// Required version (for packages)
    pub version_constraint: Option<String>,
}

/// Unified graph edge - language independent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedEdge {
    /// Unique edge ID
    pub id: String,

    /// Source node ID
    pub source: String,

    /// Target node ID
    pub target: String,

    /// Relation type
    pub edge_type: UnifiedEdgeType,

    /// Edge weight (for layouts)
    pub weight: f32,

    /// Optional label
    pub label: Option<String>,

    /// Additional detail
    pub detail: Option<String>,

    /// Is bidirectional relation
    pub bidirectional: bool,

    /// Additional metadata
    #[serde(default)]
    pub metadata: EdgeMetadata,
}

impl UnifiedEdge {
    pub fn new(source: String, target: String, edge_type: UnifiedEdgeType) -> Self {
        let id = format!("{}->{}:{:?}", source, target, edge_type);
        Self {
            id,
            source,
            target,
            edge_type,
            weight: 1.0,
            label: None,
            detail: None,
            bidirectional: false,
            metadata: EdgeMetadata::default(),
        }
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }
}
