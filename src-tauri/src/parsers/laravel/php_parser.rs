use regex::Regex;
use std::fs;

use crate::models::{Dependency, ParsedFile, SourceFile, Symbol, SymbolType};
use crate::parsers::{ParseError, ParserConfig, ParserResult};

/// Base PHP parser with common regex patterns for Laravel
pub struct PhpParser {
    namespace_regex: Regex,
    use_regex: Regex,
    class_regex: Regex,
    interface_regex: Regex,
    trait_regex: Regex,
    trait_use_regex: Regex,
    function_regex: Regex,
    method_regex: Regex,
    property_regex: Regex,
    const_regex: Regex,
}

impl PhpParser {
    pub fn new() -> Self {
        Self {
            // Match: namespace App\Http\Controllers;
            namespace_regex: Regex::new(r"(?m)^\s*namespace\s+([\w\\]+)\s*;").unwrap(),

            // Match: use App\Models\User;  or  use App\Models\User as UserModel;
            use_regex: Regex::new(
                r"(?m)^\s*use\s+([\w\\]+)(?:\s+as\s+(\w+))?\s*;",
            ).unwrap(),

            // Match: class UserController extends Controller implements Interface
            class_regex: Regex::new(
                r"(?m)^\s*(?:(abstract|final)\s+)?class\s+(\w+)(?:\s+extends\s+([\w\\]+))?(?:\s+implements\s+([\w\\,\s]+))?"
            ).unwrap(),

            // Match: interface Authenticatable
            interface_regex: Regex::new(
                r"(?m)^\s*interface\s+(\w+)(?:\s+extends\s+([\w\\,\s]+))?"
            ).unwrap(),

            // Match: trait HasFactory
            trait_regex: Regex::new(r"(?m)^\s*trait\s+(\w+)").unwrap(),

            // Match: use HasFactory, Notifiable;
            trait_use_regex: Regex::new(r"(?m)^\s*use\s+([\w\\]+(?:\s*,\s*[\w\\]+)*)\s*;").unwrap(),

            // Match: function boot() or public function index()
            function_regex: Regex::new(
                r"(?m)^\s*(?:(public|protected|private)\s+)?(?:(static)\s+)?function\s+(\w+)\s*\("
            ).unwrap(),

            // Match: public function store(Request $request)
            method_regex: Regex::new(
                r"(?m)^\s*(public|protected|private)\s+(?:(static)\s+)?function\s+(\w+)\s*\(([^)]*)\)"
            ).unwrap(),

            // Match: public $name; or protected string $email;
            property_regex: Regex::new(
                r"(?m)^\s*(public|protected|private)\s+(?:(static)\s+)?(?:(\??\w+)\s+)?\$(\w+)"
            ).unwrap(),

            // Match: const TABLE = 'users'; or public const STATUS_ACTIVE = 1;
            const_regex: Regex::new(
                r"(?m)^\s*(?:(public|protected|private)\s+)?const\s+(\w+)\s*="
            ).unwrap(),
        }
    }

    /// Parse a generic PHP file
    pub async fn parse(
        &self,
        file: &SourceFile,
        _config: &ParserConfig,
    ) -> ParserResult<ParsedFile> {
        let content = fs::read_to_string(&file.absolute_path)
            .map_err(ParseError::Io)?;

        let mut parsed = ParsedFile::new(file.clone());

        // Extract namespace
        let namespace = self.extract_namespace(&content);
        if let Some(ref ns) = namespace {
            parsed.metadata.insert(
                "namespace".to_string(),
                serde_json::Value::String(ns.clone()),
            );
        }

        // Extract use statements (imports)
        self.extract_use_statements(&content, &mut parsed);

        // Extract class definitions
        self.extract_classes(&content, &namespace, &mut parsed);

        // Extract interfaces
        self.extract_interfaces(&content, &namespace, &mut parsed);

        // Extract traits
        self.extract_traits(&content, &namespace, &mut parsed);

        // Extract functions (standalone)
        self.extract_functions(&content, &mut parsed);

        // Extract methods
        self.extract_methods(&content, &mut parsed);

        // Extract properties
        self.extract_properties(&content, &mut parsed);

        // Extract constants
        self.extract_constants(&content, &mut parsed);

        Ok(parsed)
    }

    pub fn extract_namespace(&self, content: &str) -> Option<String> {
        self.namespace_regex
            .captures(content)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    pub fn extract_use_statements(&self, content: &str, parsed: &mut ParsedFile) {
        for caps in self.use_regex.captures_iter(content) {
            let target = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let alias = caps.get(2).map(|m| m.as_str().to_string());

            if !target.is_empty() {
                parsed.add_dependency(Dependency {
                    target,
                    alias,
                    line_number: None,
                    is_interface: false,
                    is_implementation: false,
                });
            }
        }
    }

    pub fn extract_classes(
        &self,
        content: &str,
        namespace: &Option<String>,
        parsed: &mut ParsedFile,
    ) {
        for caps in self.class_regex.captures_iter(content) {
            let modifier = caps.get(1).map(|m| m.as_str());
            let class_name = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            let extends = caps.get(3).map(|m| m.as_str().to_string());
            let implements = caps.get(4).map(|m| {
                m.as_str()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            });

            if !class_name.is_empty() {
                let qualified_name = match namespace {
                    Some(ns) => format!("{}\\{}", ns, class_name),
                    None => class_name.clone(),
                };

                parsed.add_symbol(Symbol {
                    name: class_name,
                    qualified_name,
                    symbol_type: SymbolType::Class,
                    visibility: Some("public".to_string()),
                    is_abstract: Some(modifier == Some("abstract")),
                    is_static: None,
                    extends,
                    implements,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }

    pub fn extract_interfaces(
        &self,
        content: &str,
        namespace: &Option<String>,
        parsed: &mut ParsedFile,
    ) {
        for caps in self.interface_regex.captures_iter(content) {
            let iface_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let extends = caps.get(2).map(|m| m.as_str().to_string());

            if !iface_name.is_empty() {
                let qualified_name = match namespace {
                    Some(ns) => format!("{}\\{}", ns, iface_name),
                    None => iface_name.clone(),
                };

                parsed.add_symbol(Symbol {
                    name: iface_name,
                    qualified_name,
                    symbol_type: SymbolType::Interface,
                    visibility: Some("public".to_string()),
                    is_abstract: None,
                    is_static: None,
                    extends,
                    implements: None,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }

    pub fn extract_traits(
        &self,
        content: &str,
        namespace: &Option<String>,
        parsed: &mut ParsedFile,
    ) {
        for caps in self.trait_regex.captures_iter(content) {
            let trait_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();

            if !trait_name.is_empty() {
                let qualified_name = match namespace {
                    Some(ns) => format!("{}\\{}", ns, trait_name),
                    None => trait_name.clone(),
                };

                parsed.add_symbol(Symbol {
                    name: trait_name,
                    qualified_name,
                    symbol_type: SymbolType::Trait,
                    visibility: Some("public".to_string()),
                    is_abstract: None,
                    is_static: None,
                    extends: None,
                    implements: None,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }

    pub fn extract_functions(&self, content: &str, parsed: &mut ParsedFile) {
        // Only extract top-level functions (not methods inside classes)
        // This is a simplification - for standalone function files
        for caps in self.function_regex.captures_iter(content) {
            let visibility = caps.get(1).map(|m| m.as_str().to_string());
            let is_static = caps.get(2).is_some();
            let func_name = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();

            // Skip constructor/destructor and class methods (handled separately)
            if func_name.starts_with("__") || visibility.is_some() {
                continue;
            }

            if !func_name.is_empty() {
                parsed.add_symbol(Symbol {
                    name: func_name.clone(),
                    qualified_name: func_name,
                    symbol_type: SymbolType::Function,
                    visibility: None,
                    is_abstract: None,
                    is_static: Some(is_static),
                    extends: None,
                    implements: None,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }

    pub fn extract_methods(&self, content: &str, parsed: &mut ParsedFile) {
        for caps in self.method_regex.captures_iter(content) {
            let visibility = caps.get(1).map(|m| m.as_str().to_string());
            let is_static = caps.get(2).is_some();
            let method_name = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();

            if !method_name.is_empty() {
                parsed.add_symbol(Symbol {
                    name: method_name.clone(),
                    qualified_name: method_name,
                    symbol_type: SymbolType::Method,
                    visibility,
                    is_abstract: None,
                    is_static: Some(is_static),
                    extends: None,
                    implements: None,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }

    pub fn extract_properties(&self, content: &str, parsed: &mut ParsedFile) {
        for caps in self.property_regex.captures_iter(content) {
            let visibility = caps.get(1).map(|m| m.as_str().to_string());
            let is_static = caps.get(2).is_some();
            let prop_name = caps.get(4).map(|m| m.as_str().to_string()).unwrap_or_default();

            if !prop_name.is_empty() {
                parsed.add_symbol(Symbol {
                    name: prop_name.clone(),
                    qualified_name: prop_name,
                    symbol_type: SymbolType::Property,
                    visibility,
                    is_abstract: None,
                    is_static: Some(is_static),
                    extends: None,
                    implements: None,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }

    pub fn extract_constants(&self, content: &str, parsed: &mut ParsedFile) {
        for caps in self.const_regex.captures_iter(content) {
            let visibility = caps.get(1).map(|m| m.as_str().to_string());
            let const_name = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();

            if !const_name.is_empty() {
                parsed.add_symbol(Symbol {
                    name: const_name.clone(),
                    qualified_name: const_name,
                    symbol_type: SymbolType::Constant,
                    visibility,
                    is_abstract: None,
                    is_static: Some(true),
                    extends: None,
                    implements: None,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }

    /// Extract traits used inside a class
    pub fn extract_trait_uses(&self, content: &str) -> Vec<String> {
        let mut traits = Vec::new();

        // Find content inside class body
        if let Some(class_start) = content.find('{') {
            let class_content = &content[class_start..];

            for caps in self.trait_use_regex.captures_iter(class_content) {
                if let Some(trait_list) = caps.get(1) {
                    for trait_name in trait_list.as_str().split(',') {
                        let name = trait_name.trim().to_string();
                        // Avoid capturing namespace use statements
                        if !name.contains('\\') || name.starts_with("\\") {
                            continue;
                        }
                        if !name.is_empty() && !traits.contains(&name) {
                            traits.push(name);
                        }
                    }
                }
            }
        }

        traits
    }
}

impl Default for PhpParser {
    fn default() -> Self {
        Self::new()
    }
}
