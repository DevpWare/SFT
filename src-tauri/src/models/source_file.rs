use serde::{Deserialize, Serialize};

/// Represents a source file found during scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    /// File name with extension
    pub name: String,

    /// Relative path from project root
    pub path: String,

    /// Absolute path on filesystem
    pub absolute_path: String,

    /// File extension (without dot)
    pub extension: String,

    /// File size in bytes
    pub size_bytes: u64,

    /// MD5 hash for identification
    pub hash: Option<String>,

    /// Last modified timestamp
    pub modified_at: Option<String>,
}

impl SourceFile {
    pub fn new(name: String, path: String, absolute_path: String) -> Self {
        let extension = std::path::Path::new(&name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        Self {
            name,
            path,
            absolute_path,
            extension,
            size_bytes: 0,
            hash: None,
            modified_at: None,
        }
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size_bytes = size;
        self
    }

    pub fn with_hash(mut self, hash: String) -> Self {
        self.hash = Some(hash);
        self
    }

    /// Check if this is a Delphi unit file
    pub fn is_delphi_unit(&self) -> bool {
        self.extension.eq_ignore_ascii_case("pas")
    }

    /// Check if this is a Delphi form file
    pub fn is_delphi_form(&self) -> bool {
        self.extension.eq_ignore_ascii_case("dfm")
            || self.extension.eq_ignore_ascii_case("fmx")
    }

    /// Check if this is a PHP file
    pub fn is_php(&self) -> bool {
        self.extension.eq_ignore_ascii_case("php")
    }

    /// Check if this is a Blade template
    pub fn is_blade(&self) -> bool {
        self.name.ends_with(".blade.php")
    }
}
