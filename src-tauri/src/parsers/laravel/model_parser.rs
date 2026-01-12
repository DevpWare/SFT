use regex::Regex;
use std::fs;

use crate::models::{Dependency, ParsedFile, SourceFile, Symbol, SymbolType};
use crate::parsers::{ParseError, ParserConfig, ParserResult};

/// Parser for Laravel Eloquent Models
pub struct ModelParser {
    namespace_regex: Regex,
    use_regex: Regex,
    class_regex: Regex,
    property_regex: Regex,
    method_regex: Regex,
    relation_regex: Regex,
    scope_regex: Regex,
    accessor_regex: Regex,
    mutator_regex: Regex,
    cast_attribute_regex: Regex,
}

impl ModelParser {
    pub fn new() -> Self {
        Self {
            namespace_regex: Regex::new(r"(?m)^\s*namespace\s+([\w\\]+)\s*;").unwrap(),
            use_regex: Regex::new(r"(?m)^\s*use\s+([\w\\]+)(?:\s+as\s+(\w+))?\s*;").unwrap(),
            class_regex: Regex::new(
                r"(?m)^\s*class\s+(\w+)(?:\s+extends\s+([\w\\]+))?(?:\s+implements\s+([\w\\,\s]+))?"
            ).unwrap(),
            // Match: protected $fillable = [...]; or public $timestamps = false;
            property_regex: Regex::new(
                r"(?m)^\s*(public|protected|private)\s+(?:(static)\s+)?(?:\??\w+\s+)?\$(\w+)\s*=?"
            ).unwrap(),
            method_regex: Regex::new(
                r"(?m)^\s*(public|protected|private)\s+(?:(static)\s+)?function\s+(\w+)\s*\(([^)]*)\)"
            ).unwrap(),
            // Match relationship methods: return $this->hasMany(Post::class);
            relation_regex: Regex::new(
                r"\$this\s*->\s*(hasOne|hasMany|belongsTo|belongsToMany|hasManyThrough|hasOneThrough|morphOne|morphMany|morphTo|morphToMany|morphedByMany)\s*\(\s*([^)]+)\)"
            ).unwrap(),
            // Match: public function scopeActive($query)
            scope_regex: Regex::new(
                r"(?m)^\s*public\s+function\s+scope([A-Z]\w*)\s*\("
            ).unwrap(),
            // Match: public function getFullNameAttribute() - Laravel < 9 style
            accessor_regex: Regex::new(
                r"(?m)^\s*public\s+function\s+get([A-Z]\w*)Attribute\s*\("
            ).unwrap(),
            // Match: public function setPasswordAttribute($value) - Laravel < 9 style
            mutator_regex: Regex::new(
                r"(?m)^\s*public\s+function\s+set([A-Z]\w*)Attribute\s*\("
            ).unwrap(),
            // Match Laravel 9+ cast attributes: #[Attribute] or Attribute::make()
            cast_attribute_regex: Regex::new(
                r"(?m)protected\s+function\s+(\w+)\s*\(\s*\)\s*:\s*Attribute"
            ).unwrap(),
        }
    }

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

        // Extract use statements
        self.extract_use_statements(&content, &mut parsed);

        // Extract model class
        self.extract_model_class(&content, &namespace, &mut parsed);

        // Extract model properties (fillable, guarded, etc.)
        let properties = self.extract_model_properties(&content);
        if let serde_json::Value::Object(ref map) = properties {
            if !map.is_empty() {
                parsed.metadata.insert(
                    "model_properties".to_string(),
                    properties,
                );
            }
        }

        // Extract relationships
        let relationships = self.extract_relationships(&content);
        if !relationships.is_empty() {
            parsed.metadata.insert(
                "relationships".to_string(),
                serde_json::json!(relationships),
            );
        }

        // Extract scopes
        let scopes = self.extract_scopes(&content);
        if !scopes.is_empty() {
            parsed.metadata.insert(
                "scopes".to_string(),
                serde_json::json!(scopes),
            );
        }

        // Extract accessors
        let accessors = self.extract_accessors(&content);
        if !accessors.is_empty() {
            parsed.metadata.insert(
                "accessors".to_string(),
                serde_json::json!(accessors),
            );
        }

        // Extract mutators
        let mutators = self.extract_mutators(&content);
        if !mutators.is_empty() {
            parsed.metadata.insert(
                "mutators".to_string(),
                serde_json::json!(mutators),
            );
        }

        // Extract casts
        let casts = self.extract_casts(&content);
        if let serde_json::Value::Object(ref map) = casts {
            if !map.is_empty() {
                parsed.metadata.insert(
                    "casts".to_string(),
                    casts,
                );
            }
        }

        // Check for traits (SoftDeletes, HasFactory, etc.)
        let traits = self.extract_traits_used(&content);
        if !traits.is_empty() {
            parsed.metadata.insert(
                "traits_used".to_string(),
                serde_json::json!(traits),
            );
        }

        // Extract table name if specified
        if let Some(table) = self.extract_table_name(&content) {
            parsed.metadata.insert(
                "table".to_string(),
                serde_json::Value::String(table),
            );
        }

        // Extract primary key if specified
        if let Some(pk) = self.extract_primary_key(&content) {
            parsed.metadata.insert(
                "primary_key".to_string(),
                serde_json::Value::String(pk),
            );
        }

        // Extract methods
        self.extract_methods(&content, &mut parsed);

        Ok(parsed)
    }

    fn extract_namespace(&self, content: &str) -> Option<String> {
        self.namespace_regex
            .captures(content)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn extract_use_statements(&self, content: &str, parsed: &mut ParsedFile) {
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

    fn extract_model_class(
        &self,
        content: &str,
        namespace: &Option<String>,
        parsed: &mut ParsedFile,
    ) {
        if let Some(caps) = self.class_regex.captures(content) {
            let class_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let extends = caps.get(2).map(|m| m.as_str().to_string());
            let implements = caps.get(3).map(|m| {
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
                    is_abstract: None,
                    is_static: None,
                    extends,
                    implements,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }

    fn extract_model_properties(&self, content: &str) -> serde_json::Value {
        let mut properties = serde_json::Map::new();

        // Extract $fillable array
        if let Some(fillable) = self.extract_array_property(content, "fillable") {
            properties.insert("fillable".to_string(), serde_json::json!(fillable));
        }

        // Extract $guarded array
        if let Some(guarded) = self.extract_array_property(content, "guarded") {
            properties.insert("guarded".to_string(), serde_json::json!(guarded));
        }

        // Extract $hidden array
        if let Some(hidden) = self.extract_array_property(content, "hidden") {
            properties.insert("hidden".to_string(), serde_json::json!(hidden));
        }

        // Extract $visible array
        if let Some(visible) = self.extract_array_property(content, "visible") {
            properties.insert("visible".to_string(), serde_json::json!(visible));
        }

        // Extract $appends array
        if let Some(appends) = self.extract_array_property(content, "appends") {
            properties.insert("appends".to_string(), serde_json::json!(appends));
        }

        // Extract $with array (eager loading)
        if let Some(with) = self.extract_array_property(content, "with") {
            properties.insert("with".to_string(), serde_json::json!(with));
        }

        // Extract $dates array (deprecated in Laravel 8+)
        if let Some(dates) = self.extract_array_property(content, "dates") {
            properties.insert("dates".to_string(), serde_json::json!(dates));
        }

        // Check for timestamps
        let timestamps_regex = Regex::new(r"\$timestamps\s*=\s*(true|false)").unwrap();
        if let Some(caps) = timestamps_regex.captures(content) {
            let value = caps.get(1).map(|m| m.as_str() == "true").unwrap_or(true);
            properties.insert("timestamps".to_string(), serde_json::json!(value));
        }

        // Check for incrementing
        let incrementing_regex = Regex::new(r"\$incrementing\s*=\s*(true|false)").unwrap();
        if let Some(caps) = incrementing_regex.captures(content) {
            let value = caps.get(1).map(|m| m.as_str() == "true").unwrap_or(true);
            properties.insert("incrementing".to_string(), serde_json::json!(value));
        }

        serde_json::Value::Object(properties)
    }

    fn extract_array_property(&self, content: &str, property_name: &str) -> Option<Vec<String>> {
        let pattern = format!(
            r"\${}\s*=\s*\[([^\]]*)\]",
            regex::escape(property_name)
        );
        let regex = Regex::new(&pattern).ok()?;

        regex.captures(content).and_then(|caps| {
            caps.get(1).map(|m| {
                m.as_str()
                    .split(',')
                    .filter_map(|s| {
                        let trimmed = s.trim().trim_matches(|c| c == '\'' || c == '"');
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed.to_string())
                        }
                    })
                    .collect()
            })
        })
    }

    fn extract_relationships(&self, content: &str) -> Vec<serde_json::Value> {
        let mut relationships = Vec::new();

        // First, find all relationship method definitions
        let method_regex = Regex::new(
            r"(?s)public\s+function\s+(\w+)\s*\([^)]*\)\s*(?::\s*[\w\\]+)?\s*\{([^}]+)\}"
        ).unwrap();

        for method_caps in method_regex.captures_iter(content) {
            let method_name = method_caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let method_body = method_caps.get(2).map(|m| m.as_str()).unwrap_or("");

            // Check if this method contains a relationship call
            for rel_caps in self.relation_regex.captures_iter(method_body) {
                let rel_type = rel_caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let rel_args = rel_caps.get(2).map(|m| m.as_str()).unwrap_or("");

                // Extract the related model from arguments
                let related_model = self.extract_related_model(rel_args);

                relationships.push(serde_json::json!({
                    "method": method_name,
                    "type": rel_type,
                    "related_model": related_model,
                    "raw_args": rel_args.trim()
                }));
            }
        }

        relationships
    }

    fn extract_related_model(&self, args: &str) -> Option<String> {
        // Match Model::class or 'App\Models\Model' or "App\Models\Model"
        let class_regex = Regex::new(r#"([A-Z]\w*)::class|['"]([^'"]+)['"]"#).unwrap();

        class_regex.captures(args).and_then(|caps| {
            caps.get(1)
                .or_else(|| caps.get(2))
                .map(|m| m.as_str().to_string())
        })
    }

    fn extract_scopes(&self, content: &str) -> Vec<String> {
        let mut scopes = Vec::new();

        for caps in self.scope_regex.captures_iter(content) {
            if let Some(scope_name) = caps.get(1) {
                scopes.push(scope_name.as_str().to_string());
            }
        }

        scopes
    }

    fn extract_accessors(&self, content: &str) -> Vec<String> {
        let mut accessors = Vec::new();

        // Laravel < 9 style: getXxxAttribute
        for caps in self.accessor_regex.captures_iter(content) {
            if let Some(attr_name) = caps.get(1) {
                accessors.push(self.snake_case(attr_name.as_str()));
            }
        }

        // Laravel 9+ style: protected function xxx(): Attribute
        for caps in self.cast_attribute_regex.captures_iter(content) {
            if let Some(attr_name) = caps.get(1) {
                let name = attr_name.as_str().to_string();
                if !accessors.contains(&name) {
                    accessors.push(name);
                }
            }
        }

        accessors
    }

    fn extract_mutators(&self, content: &str) -> Vec<String> {
        let mut mutators = Vec::new();

        for caps in self.mutator_regex.captures_iter(content) {
            if let Some(attr_name) = caps.get(1) {
                mutators.push(self.snake_case(attr_name.as_str()));
            }
        }

        mutators
    }

    fn extract_casts(&self, content: &str) -> serde_json::Value {
        // Match $casts = ['field' => 'type', ...];
        let casts_regex = Regex::new(r"\$casts\s*=\s*\[([^\]]+)\]").unwrap();

        if let Some(caps) = casts_regex.captures(content) {
            if let Some(casts_content) = caps.get(1) {
                let mut casts = serde_json::Map::new();
                let pair_regex = Regex::new(r#"['"](\w+)['"]\s*=>\s*['"]([^'"]+)['"]"#).unwrap();

                for pair_caps in pair_regex.captures_iter(casts_content.as_str()) {
                    let field = pair_caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    let cast_type = pair_caps.get(2).map(|m| m.as_str()).unwrap_or("");

                    if !field.is_empty() && !cast_type.is_empty() {
                        casts.insert(field.to_string(), serde_json::json!(cast_type));
                    }
                }

                return serde_json::Value::Object(casts);
            }
        }

        // Also check for casts() method (Laravel 9+)
        let casts_method_regex = Regex::new(
            r"(?s)protected\s+function\s+casts\s*\(\s*\)\s*:\s*array\s*\{[^}]*return\s*\[([^\]]+)\]"
        ).unwrap();

        if let Some(caps) = casts_method_regex.captures(content) {
            if let Some(casts_content) = caps.get(1) {
                let mut casts = serde_json::Map::new();
                let pair_regex = Regex::new(r#"['"](\w+)['"]\s*=>\s*([^,\]]+)"#).unwrap();

                for pair_caps in pair_regex.captures_iter(casts_content.as_str()) {
                    let field = pair_caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    let cast_type = pair_caps.get(2).map(|m| m.as_str().trim().trim_matches(|c| c == '\'' || c == '"')).unwrap_or("");

                    if !field.is_empty() && !cast_type.is_empty() {
                        casts.insert(field.to_string(), serde_json::json!(cast_type));
                    }
                }

                return serde_json::Value::Object(casts);
            }
        }

        serde_json::Value::Object(serde_json::Map::new())
    }

    fn extract_traits_used(&self, content: &str) -> Vec<String> {
        let mut traits = Vec::new();

        // Match: use HasFactory, SoftDeletes, Notifiable;
        let trait_use_regex = Regex::new(
            r"(?m)^\s*use\s+((?:[\w\\]+\s*,\s*)*[\w\\]+)\s*;"
        ).unwrap();

        // Find class body start
        if let Some(class_start) = content.find("class ") {
            if let Some(brace_pos) = content[class_start..].find('{') {
                let class_body = &content[class_start + brace_pos..];

                for caps in trait_use_regex.captures_iter(class_body) {
                    if let Some(trait_list) = caps.get(1) {
                        for trait_name in trait_list.as_str().split(',') {
                            let name = trait_name.trim();
                            // Filter out namespace imports (they contain backslashes at start)
                            if !name.is_empty() && !name.starts_with('\\') {
                                // Get just the trait name without namespace
                                let short_name = name.rsplit('\\').next().unwrap_or(name);
                                if !traits.contains(&short_name.to_string()) {
                                    traits.push(short_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        traits
    }

    fn extract_table_name(&self, content: &str) -> Option<String> {
        let table_regex = Regex::new(r#"\$table\s*=\s*['"]([^'"]+)['"]"#).unwrap();

        table_regex.captures(content)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn extract_primary_key(&self, content: &str) -> Option<String> {
        let pk_regex = Regex::new(r#"\$primaryKey\s*=\s*['"]([^'"]+)['"]"#).unwrap();

        pk_regex.captures(content)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn extract_methods(&self, content: &str, parsed: &mut ParsedFile) {
        for caps in self.method_regex.captures_iter(content) {
            let visibility = caps.get(1).map(|m| m.as_str().to_string());
            let is_static = caps.get(2).is_some();
            let method_name = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();

            // Skip magic methods
            if method_name.starts_with("__") {
                continue;
            }

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

    /// Convert PascalCase to snake_case
    fn snake_case(&self, s: &str) -> String {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 {
                    result.push('_');
                }
                result.push(c.to_lowercase().next().unwrap());
            } else {
                result.push(c);
            }
        }
        result
    }
}

impl Default for ModelParser {
    fn default() -> Self {
        Self::new()
    }
}
