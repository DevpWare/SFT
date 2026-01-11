use regex::Regex;
use std::fs;

use crate::models::{ParsedFile, SourceFile, Symbol, SymbolType};
use crate::parsers::{ParserConfig, ParserResult, ParseError};

/// Parser for Delphi .dfm/.fmx form files
pub struct DfmParser {
    object_regex: Regex,
    property_regex: Regex,
}

impl DfmParser {
    pub fn new() -> Self {
        Self {
            // Match: object ComponentName: TComponentClass
            object_regex: Regex::new(
                r"(?i)^\s*(?:object|inherited)\s+(\w+)\s*:\s*(\w+)"
            ).unwrap(),

            // Match: PropertyName = Value
            property_regex: Regex::new(
                r"^\s*(\w+)\s*=\s*(.+?)$"
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

        // Extract all components
        self.extract_components(&content, &mut parsed);

        Ok(parsed)
    }

    fn extract_components(&self, content: &str, parsed: &mut ParsedFile) {
        let mut depth: u32 = 0;
        let mut current_component: Option<String> = None;

        for line in content.lines() {
            let trimmed = line.trim();

            // Check for object declaration
            if let Some(caps) = self.object_regex.captures(trimmed) {
                let component_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                let component_type = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();

                parsed.add_symbol(Symbol {
                    name: component_name.clone(),
                    qualified_name: format!("{}: {}", component_name, component_type),
                    symbol_type: SymbolType::Property, // Using Property for components
                    visibility: None,
                    is_abstract: None,
                    is_static: None,
                    extends: Some(component_type),
                    implements: None,
                    line_start: None,
                    line_end: None,
                });

                current_component = Some(component_name);
                depth += 1;
            }

            // Check for end of object
            if trimmed.eq_ignore_ascii_case("end") {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    current_component = None;
                }
            }

            // Extract SQL queries from components (common in Delphi data modules)
            if let Some(_comp) = &current_component {
                if trimmed.starts_with("SQL.Strings") || trimmed.starts_with("CommandText") {
                    // This is a SQL property, could extract the query
                    // For now, just note it exists
                }
            }
        }
    }
}

impl Default for DfmParser {
    fn default() -> Self {
        Self::new()
    }
}
