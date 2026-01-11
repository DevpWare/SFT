use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::SourceFile;

/// Represents a symbol found in code (class, function, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// Symbol name
    pub name: String,

    /// Fully qualified name
    pub qualified_name: String,

    /// Symbol type (class, function, interface, etc.)
    pub symbol_type: SymbolType,

    /// Visibility (public, private, protected)
    pub visibility: Option<String>,

    /// Is abstract
    pub is_abstract: Option<bool>,

    /// Is static
    pub is_static: Option<bool>,

    /// Parent class (for inheritance)
    pub extends: Option<String>,

    /// Implemented interfaces
    pub implements: Option<Vec<String>>,

    /// Start line
    pub line_start: Option<u32>,

    /// End line
    pub line_end: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SymbolType {
    Class,
    Interface,
    Trait,
    Enum,
    Function,
    Method,
    Variable,
    Constant,
    Property,
    Record,
    Unit,
}

/// Represents a dependency found in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Target name (what is being used/imported)
    pub target: String,

    /// Alias if any (use X as Y)
    pub alias: Option<String>,

    /// Line number where dependency is declared
    pub line_number: Option<u32>,

    /// Is from interface section (Delphi)
    pub is_interface: bool,

    /// Is from implementation section (Delphi)
    pub is_implementation: bool,
}

/// Result of parsing a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    /// Original source file
    pub source: SourceFile,

    /// Symbols found (classes, functions, etc.)
    pub symbols: Vec<Symbol>,

    /// Dependencies (imports, uses, require)
    pub dependencies: Vec<Dependency>,

    /// Language-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Non-fatal parsing warnings
    pub warnings: Vec<String>,
}

impl ParsedFile {
    pub fn new(source: SourceFile) -> Self {
        Self {
            source,
            symbols: Vec::new(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }

    pub fn add_dependency(&mut self, dep: Dependency) {
        self.dependencies.push(dep);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Result of parsing an entire project
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParseResult {
    /// All parsed files
    pub files: Vec<ParsedFile>,

    /// Files that failed to parse
    pub errors: HashMap<String, String>,

    /// Total files processed
    pub total_processed: usize,

    /// Total files with errors
    pub total_errors: usize,
}

impl ParseResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_parsed_file(&mut self, file: ParsedFile) {
        self.total_processed += 1;
        self.files.push(file);
    }

    pub fn add_error(&mut self, path: String, error: String) {
        self.total_processed += 1;
        self.total_errors += 1;
        self.errors.insert(path, error);
    }
}
