use serde::{Deserialize, Serialize};

/// Supported project types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    /// Delphi/Object Pascal projects
    Delphi,

    /// Laravel PHP framework
    Laravel,

    /// Node.js / TypeScript projects
    NodeJs,

    /// Generic PHP (non-Laravel)
    Php,

    /// C# / .NET projects
    CSharp,

    /// Java projects
    Java,

    /// Python projects
    Python,

    /// Go projects
    Go,

    /// Rust projects
    RustLang,

    /// Unknown/unsupported project type
    Unknown,
}

impl Default for ProjectType {
    fn default() -> Self {
        ProjectType::Unknown
    }
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Delphi => write!(f, "Delphi"),
            ProjectType::Laravel => write!(f, "Laravel (PHP)"),
            ProjectType::NodeJs => write!(f, "Node.js / TypeScript"),
            ProjectType::Php => write!(f, "PHP"),
            ProjectType::CSharp => write!(f, "C# (.NET)"),
            ProjectType::Java => write!(f, "Java"),
            ProjectType::Python => write!(f, "Python"),
            ProjectType::Go => write!(f, "Go"),
            ProjectType::RustLang => write!(f, "Rust"),
            ProjectType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl ProjectType {
    /// Get file extensions associated with this project type
    pub fn file_extensions(&self) -> Vec<&'static str> {
        match self {
            ProjectType::Delphi => vec!["pas", "dfm", "fmx", "dpr", "dpk", "dproj"],
            ProjectType::Laravel | ProjectType::Php => vec!["php"],
            ProjectType::NodeJs => vec!["js", "ts", "jsx", "tsx", "mjs", "cjs"],
            ProjectType::CSharp => vec!["cs", "csx"],
            ProjectType::Java => vec!["java"],
            ProjectType::Python => vec!["py", "pyw"],
            ProjectType::Go => vec!["go"],
            ProjectType::RustLang => vec!["rs"],
            ProjectType::Unknown => vec![],
        }
    }

    /// Get primary color for this project type (hex)
    pub fn primary_color(&self) -> &'static str {
        match self {
            ProjectType::Delphi => "#E31D1D",
            ProjectType::Laravel => "#FF2D20",
            ProjectType::Php => "#777BB4",
            ProjectType::NodeJs => "#339933",
            ProjectType::CSharp => "#512BD4",
            ProjectType::Java => "#007396",
            ProjectType::Python => "#3776AB",
            ProjectType::Go => "#00ADD8",
            ProjectType::RustLang => "#DEA584",
            ProjectType::Unknown => "#6B7280",
        }
    }
}
