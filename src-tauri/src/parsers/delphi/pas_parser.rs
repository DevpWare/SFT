use regex::Regex;
use std::fs;

use crate::models::{Dependency, ParsedFile, SourceFile, Symbol, SymbolType};
use crate::parsers::{ParserConfig, ParserResult, ParseError};

/// Parser for Delphi .pas files
pub struct PasParser {
    unit_regex: Regex,
    uses_regex: Regex,
    class_regex: Regex,
    interface_regex: Regex,
    procedure_regex: Regex,
    function_regex: Regex,
}

impl PasParser {
    pub fn new() -> Self {
        Self {
            // Match: unit UnitName;
            unit_regex: Regex::new(r"(?i)^\s*unit\s+(\w+)\s*;").unwrap(),

            // Match: uses clause (captures everything between uses and ;)
            uses_regex: Regex::new(r"(?is)uses\s+(.*?);").unwrap(),

            // Match: TClassName = class(TParent)
            class_regex: Regex::new(
                r"(?i)(\w+)\s*=\s*class\s*(?:\((\w+)\))?"
            ).unwrap(),

            // Match: IInterfaceName = interface
            interface_regex: Regex::new(
                r"(?i)(\w+)\s*=\s*interface\s*(?:\[|(?:\((\w+)\)))?"
            ).unwrap(),

            // Match: procedure Name
            procedure_regex: Regex::new(
                r"(?i)^\s*(?:(class)\s+)?procedure\s+(\w+)(?:\.(\w+))?\s*(?:\(|;)"
            ).unwrap(),

            // Match: function Name
            function_regex: Regex::new(
                r"(?i)^\s*(?:(class)\s+)?function\s+(\w+)(?:\.(\w+))?\s*(?:\(|:)"
            ).unwrap(),
        }
    }

    pub async fn parse(
        &self,
        file: &SourceFile,
        _config: &ParserConfig,
    ) -> ParserResult<ParsedFile> {
        let content = fs::read_to_string(&file.absolute_path)
            .map_err(|e| ParseError::Io(e))?;

        let mut parsed = ParsedFile::new(file.clone());

        // Extract unit name
        if let Some(caps) = self.unit_regex.captures(&content) {
            let unit_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            parsed.add_symbol(Symbol {
                name: unit_name.clone(),
                qualified_name: unit_name,
                symbol_type: SymbolType::Unit,
                visibility: Some("public".to_string()),
                is_abstract: None,
                is_static: None,
                extends: None,
                implements: None,
                line_start: Some(1),
                line_end: None,
            });
        }

        // Extract uses clauses
        self.extract_uses(&content, &mut parsed);

        // Extract classes
        self.extract_classes(&content, &mut parsed);

        // Extract interfaces
        self.extract_interfaces(&content, &mut parsed);

        // Extract procedures and functions
        self.extract_procedures(&content, &mut parsed);
        self.extract_functions(&content, &mut parsed);

        Ok(parsed)
    }

    fn extract_uses(&self, content: &str, parsed: &mut ParsedFile) {
        // Find interface section
        let interface_pos = content.to_lowercase().find("interface");
        let implementation_pos = content.to_lowercase().find("implementation");

        for caps in self.uses_regex.captures_iter(content) {
            if let Some(uses_match) = caps.get(1) {
                let uses_str = uses_match.as_str();
                let match_pos = uses_match.start();

                // Determine if this is interface or implementation uses
                let is_interface = match (interface_pos, implementation_pos) {
                    (Some(iface), Some(impl_)) => match_pos > iface && match_pos < impl_,
                    (Some(iface), None) => match_pos > iface,
                    _ => false,
                };

                let is_implementation = match implementation_pos {
                    Some(impl_) => match_pos > impl_,
                    None => false,
                };

                // Parse individual unit names
                for unit in uses_str.split(',') {
                    let unit_name = unit
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();

                    if !unit_name.is_empty() && unit_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        parsed.add_dependency(Dependency {
                            target: unit_name,
                            alias: None,
                            line_number: None,
                            is_interface,
                            is_implementation,
                        });
                    }
                }
            }
        }
    }

    fn extract_classes(&self, content: &str, parsed: &mut ParsedFile) {
        for caps in self.class_regex.captures_iter(content) {
            let class_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let parent = caps.get(2).map(|m| m.as_str().to_string());

            if !class_name.is_empty() && class_name.starts_with('T') {
                parsed.add_symbol(Symbol {
                    name: class_name.clone(),
                    qualified_name: class_name,
                    symbol_type: SymbolType::Class,
                    visibility: Some("public".to_string()),
                    is_abstract: None,
                    is_static: None,
                    extends: parent,
                    implements: None,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }

    fn extract_interfaces(&self, content: &str, parsed: &mut ParsedFile) {
        for caps in self.interface_regex.captures_iter(content) {
            let iface_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let parent = caps.get(2).map(|m| m.as_str().to_string());

            if !iface_name.is_empty() && iface_name.starts_with('I') {
                parsed.add_symbol(Symbol {
                    name: iface_name.clone(),
                    qualified_name: iface_name,
                    symbol_type: SymbolType::Interface,
                    visibility: Some("public".to_string()),
                    is_abstract: None,
                    is_static: None,
                    extends: parent,
                    implements: None,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }

    fn extract_procedures(&self, content: &str, parsed: &mut ParsedFile) {
        for caps in self.procedure_regex.captures_iter(content) {
            let is_class = caps.get(1).is_some();
            let name = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            let method_name = caps.get(3).map(|m| m.as_str().to_string());

            let full_name = if let Some(method) = method_name {
                format!("{}.{}", name, method)
            } else {
                name.clone()
            };

            if !full_name.is_empty() {
                parsed.add_symbol(Symbol {
                    name: full_name.clone(),
                    qualified_name: full_name,
                    symbol_type: SymbolType::Method,
                    visibility: None,
                    is_abstract: None,
                    is_static: Some(is_class),
                    extends: None,
                    implements: None,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }

    fn extract_functions(&self, content: &str, parsed: &mut ParsedFile) {
        for caps in self.function_regex.captures_iter(content) {
            let is_class = caps.get(1).is_some();
            let name = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            let method_name = caps.get(3).map(|m| m.as_str().to_string());

            let full_name = if let Some(method) = method_name {
                format!("{}.{}", name, method)
            } else {
                name.clone()
            };

            if !full_name.is_empty() {
                parsed.add_symbol(Symbol {
                    name: full_name.clone(),
                    qualified_name: full_name,
                    symbol_type: SymbolType::Function,
                    visibility: None,
                    is_abstract: None,
                    is_static: Some(is_class),
                    extends: None,
                    implements: None,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }
}

impl Default for PasParser {
    fn default() -> Self {
        Self::new()
    }
}
