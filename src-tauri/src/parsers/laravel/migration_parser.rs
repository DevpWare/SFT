use regex::Regex;
use std::fs;

use crate::models::{Dependency, ParsedFile, SourceFile, Symbol, SymbolType};
use crate::parsers::{ParseError, ParserConfig, ParserResult};

/// Parser for Laravel database migrations
pub struct MigrationParser {
    // Class and method patterns
    class_regex: Regex,
    up_method_regex: Regex,
    down_method_regex: Regex,

    // Schema operations
    create_table_regex: Regex,
    table_modify_regex: Regex,
    drop_table_regex: Regex,
    rename_table_regex: Regex,

    // Column definitions
    column_regex: Regex,
    column_type_regex: Regex,

    // Index and key patterns
    primary_key_regex: Regex,
    unique_regex: Regex,
    index_regex: Regex,
    foreign_regex: Regex,

    // Common shorthand columns
    timestamps_regex: Regex,
    soft_deletes_regex: Regex,
    remember_token_regex: Regex,
}

impl MigrationParser {
    pub fn new() -> Self {
        Self {
            // Match: class CreateUsersTable extends Migration
            class_regex: Regex::new(
                r"class\s+(\w+)\s+extends\s+Migration"
            ).unwrap(),

            // Match: public function up()
            up_method_regex: Regex::new(
                r"(?s)public\s+function\s+up\s*\(\s*\)(?:\s*:\s*void)?\s*\{(.*?)\n\s{4}\}"
            ).unwrap(),

            // Match: public function down()
            down_method_regex: Regex::new(
                r"(?s)public\s+function\s+down\s*\(\s*\)(?:\s*:\s*void)?\s*\{(.*?)\n\s{4}\}"
            ).unwrap(),

            // Match: Schema::create('users', function (Blueprint $table) {
            create_table_regex: Regex::new(
                r#"Schema::create\s*\(\s*['"](\w+)['"]"#
            ).unwrap(),

            // Match: Schema::table('users', function (Blueprint $table) {
            table_modify_regex: Regex::new(
                r#"Schema::table\s*\(\s*['"](\w+)['"]"#
            ).unwrap(),

            // Match: Schema::dropIfExists('users') or Schema::drop('users')
            drop_table_regex: Regex::new(
                r#"Schema::(?:dropIfExists|drop)\s*\(\s*['"](\w+)['"]"#
            ).unwrap(),

            // Match: Schema::rename('old', 'new')
            rename_table_regex: Regex::new(
                r#"Schema::rename\s*\(\s*['"](\w+)['"]\s*,\s*['"](\w+)['"]"#
            ).unwrap(),

            // Match: $table->string('name') or $table->integer('age', ...)
            column_regex: Regex::new(
                r#"\$table\s*->\s*(\w+)\s*\(\s*['"](\w+)['"]"#
            ).unwrap(),

            // Column types for validation
            column_type_regex: Regex::new(
                r"(bigIncrements|bigInteger|binary|boolean|char|date|dateTime|dateTimeTz|decimal|double|enum|float|foreignId|foreignIdFor|foreignUlid|foreignUuid|geometry|id|increments|integer|ipAddress|json|jsonb|longText|macAddress|mediumIncrements|mediumInteger|mediumText|morphs|nullableMorphs|nullableTimestamps|nullableUlidMorphs|nullableUuidMorphs|point|polygon|rememberToken|set|smallIncrements|smallInteger|softDeletes|softDeletesTz|string|text|time|timeTz|timestamp|timestampTz|timestamps|timestampsTz|tinyIncrements|tinyInteger|tinyText|unsignedBigInteger|unsignedDecimal|unsignedInteger|unsignedMediumInteger|unsignedSmallInteger|unsignedTinyInteger|ulidMorphs|uuid|uuidMorphs|year)"
            ).unwrap(),

            // Match: $table->primary('id') or $table->primary(['id', 'name'])
            primary_key_regex: Regex::new(
                r#"\$table\s*->\s*primary\s*\(\s*(?:['"](\w+)['"]|\[([^\]]+)\])"#
            ).unwrap(),

            // Match: $table->unique('email') or $table->unique(['first', 'last'])
            unique_regex: Regex::new(
                r#"\$table\s*->\s*unique\s*\(\s*(?:['"](\w+)['"]|\[([^\]]+)\])"#
            ).unwrap(),

            // Match: $table->index('state') or $table->index(['account_id', 'created_at'])
            index_regex: Regex::new(
                r#"\$table\s*->\s*index\s*\(\s*(?:['"](\w+)['"]|\[([^\]]+)\])"#
            ).unwrap(),

            // Match: $table->foreign('user_id')->references('id')->on('users')
            foreign_regex: Regex::new(
                r#"\$table\s*->\s*foreign\s*\(\s*['"](\w+)['"]\s*\)\s*->\s*references\s*\(\s*['"](\w+)['"]\s*\)\s*->\s*on\s*\(\s*['"](\w+)['"]"#
            ).unwrap(),

            // Match: $table->timestamps()
            timestamps_regex: Regex::new(
                r"\$table\s*->\s*timestamps\s*\("
            ).unwrap(),

            // Match: $table->softDeletes()
            soft_deletes_regex: Regex::new(
                r"\$table\s*->\s*softDeletes(?:Tz)?\s*\("
            ).unwrap(),

            // Match: $table->rememberToken()
            remember_token_regex: Regex::new(
                r"\$table\s*->\s*rememberToken\s*\("
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

        // Extract migration class name
        let class_name = self.extract_class_name(&content);
        if let Some(ref name) = class_name {
            parsed.add_symbol(Symbol {
                name: name.clone(),
                qualified_name: name.clone(),
                symbol_type: SymbolType::Class,
                visibility: Some("public".to_string()),
                is_abstract: None,
                is_static: None,
                extends: Some("Migration".to_string()),
                implements: None,
                line_start: None,
                line_end: None,
            });

            parsed.metadata.insert(
                "migration_class".to_string(),
                serde_json::Value::String(name.clone()),
            );
        }

        // Extract migration timestamp from filename
        if let Some(timestamp) = self.extract_migration_timestamp(&file.name) {
            parsed.metadata.insert(
                "migration_timestamp".to_string(),
                serde_json::Value::String(timestamp),
            );
        }

        // Parse up() method
        if let Some(up_content) = self.extract_up_method(&content) {
            let up_operations = self.parse_schema_operations(&up_content);
            parsed.metadata.insert(
                "up_operations".to_string(),
                serde_json::json!(up_operations),
            );
        }

        // Parse down() method
        if let Some(down_content) = self.extract_down_method(&content) {
            let down_operations = self.parse_schema_operations(&down_content);
            parsed.metadata.insert(
                "down_operations".to_string(),
                serde_json::json!(down_operations),
            );
        }

        // Extract tables created
        let tables_created = self.extract_created_tables(&content);
        if !tables_created.is_empty() {
            parsed.metadata.insert(
                "tables_created".to_string(),
                serde_json::json!(tables_created),
            );
        }

        // Extract tables modified
        let tables_modified = self.extract_modified_tables(&content);
        if !tables_modified.is_empty() {
            parsed.metadata.insert(
                "tables_modified".to_string(),
                serde_json::json!(tables_modified),
            );
        }

        // Extract tables dropped
        let tables_dropped = self.extract_dropped_tables(&content);
        if !tables_dropped.is_empty() {
            parsed.metadata.insert(
                "tables_dropped".to_string(),
                serde_json::json!(tables_dropped),
            );
        }

        // Extract all columns defined
        let columns = self.extract_all_columns(&content);
        if !columns.is_empty() {
            parsed.metadata.insert(
                "columns".to_string(),
                serde_json::json!(columns),
            );
        }

        // Extract foreign keys
        let foreign_keys = self.extract_foreign_keys(&content);
        if !foreign_keys.is_empty() {
            parsed.metadata.insert(
                "foreign_keys".to_string(),
                serde_json::json!(foreign_keys),
            );
        }

        // Extract indexes
        let indexes = self.extract_indexes(&content);
        if !indexes.is_empty() {
            parsed.metadata.insert(
                "indexes".to_string(),
                serde_json::json!(indexes),
            );
        }

        // Check for common patterns
        let has_timestamps = self.timestamps_regex.is_match(&content);
        let has_soft_deletes = self.soft_deletes_regex.is_match(&content);
        let has_remember_token = self.remember_token_regex.is_match(&content);

        parsed.metadata.insert(
            "has_timestamps".to_string(),
            serde_json::json!(has_timestamps),
        );
        parsed.metadata.insert(
            "has_soft_deletes".to_string(),
            serde_json::json!(has_soft_deletes),
        );
        parsed.metadata.insert(
            "has_remember_token".to_string(),
            serde_json::json!(has_remember_token),
        );

        // Add use statements as dependencies
        self.extract_use_statements(&content, &mut parsed);

        Ok(parsed)
    }

    fn extract_class_name(&self, content: &str) -> Option<String> {
        self.class_regex.captures(content)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn extract_migration_timestamp(&self, filename: &str) -> Option<String> {
        let timestamp_regex = Regex::new(r"^(\d{4}_\d{2}_\d{2}_\d{6})").ok()?;

        timestamp_regex.captures(filename)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn extract_up_method(&self, content: &str) -> Option<String> {
        self.up_method_regex.captures(content)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn extract_down_method(&self, content: &str) -> Option<String> {
        self.down_method_regex.captures(content)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn parse_schema_operations(&self, method_content: &str) -> Vec<serde_json::Value> {
        let mut operations = Vec::new();

        // Create table operations
        for caps in self.create_table_regex.captures_iter(method_content) {
            if let Some(table) = caps.get(1) {
                operations.push(serde_json::json!({
                    "type": "create",
                    "table": table.as_str()
                }));
            }
        }

        // Modify table operations
        for caps in self.table_modify_regex.captures_iter(method_content) {
            if let Some(table) = caps.get(1) {
                operations.push(serde_json::json!({
                    "type": "modify",
                    "table": table.as_str()
                }));
            }
        }

        // Drop table operations
        for caps in self.drop_table_regex.captures_iter(method_content) {
            if let Some(table) = caps.get(1) {
                operations.push(serde_json::json!({
                    "type": "drop",
                    "table": table.as_str()
                }));
            }
        }

        // Rename table operations
        for caps in self.rename_table_regex.captures_iter(method_content) {
            let from = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let to = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            operations.push(serde_json::json!({
                "type": "rename",
                "from": from,
                "to": to
            }));
        }

        operations
    }

    fn extract_created_tables(&self, content: &str) -> Vec<String> {
        self.create_table_regex.captures_iter(content)
            .filter_map(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn extract_modified_tables(&self, content: &str) -> Vec<String> {
        self.table_modify_regex.captures_iter(content)
            .filter_map(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn extract_dropped_tables(&self, content: &str) -> Vec<String> {
        self.drop_table_regex.captures_iter(content)
            .filter_map(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn extract_all_columns(&self, content: &str) -> Vec<serde_json::Value> {
        let mut columns = Vec::new();

        for caps in self.column_regex.captures_iter(content) {
            let column_type = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let column_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            // Validate this is a real column type
            if !self.is_valid_column_type(column_type) {
                continue;
            }

            // Get the full line for modifier extraction
            let match_start = caps.get(0).map(|m| m.start()).unwrap_or(0);
            let line_context = self.get_line_context(content, match_start);

            let modifiers = self.extract_column_modifiers(&line_context);
            let nullable = modifiers.contains(&"nullable".to_string());
            let default = self.extract_default_value(&line_context);

            columns.push(serde_json::json!({
                "name": column_name,
                "type": column_type,
                "nullable": nullable,
                "default": default,
                "modifiers": modifiers
            }));
        }

        columns
    }

    fn is_valid_column_type(&self, column_type: &str) -> bool {
        self.column_type_regex.is_match(column_type)
    }

    fn get_line_context(&self, content: &str, start: usize) -> String {
        let remaining = &content[start..];
        let end = remaining.find(';').unwrap_or(remaining.len());
        remaining[..end].to_string()
    }

    fn extract_column_modifiers(&self, line: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        let modifier_patterns = [
            ("nullable", r"->\s*nullable\s*\("),
            ("unique", r"->\s*unique\s*\("),
            ("primary", r"->\s*primary\s*\("),
            ("unsigned", r"->\s*unsigned\s*\("),
            ("autoIncrement", r"->\s*autoIncrement\s*\("),
            ("index", r"->\s*index\s*\("),
            ("useCurrent", r"->\s*useCurrent\s*\("),
            ("useCurrentOnUpdate", r"->\s*useCurrentOnUpdate\s*\("),
            ("comment", r"->\s*comment\s*\("),
            ("after", r"->\s*after\s*\("),
            ("first", r"->\s*first\s*\("),
            ("change", r"->\s*change\s*\("),
        ];

        for (name, pattern) in modifier_patterns {
            if Regex::new(pattern).map(|r| r.is_match(line)).unwrap_or(false) {
                modifiers.push(name.to_string());
            }
        }

        modifiers
    }

    fn extract_default_value(&self, line: &str) -> Option<String> {
        let default_regex = Regex::new(r"->\s*default\s*\(\s*([^)]+)\s*\)").ok()?;

        default_regex.captures(line)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
    }

    fn extract_foreign_keys(&self, content: &str) -> Vec<serde_json::Value> {
        let mut foreign_keys = Vec::new();

        for caps in self.foreign_regex.captures_iter(content) {
            let column = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let references = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let on_table = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            // Get additional context for onDelete/onUpdate
            let match_start = caps.get(0).map(|m| m.start()).unwrap_or(0);
            let context = self.get_line_context(content, match_start);

            let on_delete = self.extract_on_action(&context, "onDelete");
            let on_update = self.extract_on_action(&context, "onUpdate");

            foreign_keys.push(serde_json::json!({
                "column": column,
                "references": references,
                "on_table": on_table,
                "on_delete": on_delete,
                "on_update": on_update
            }));
        }

        // Also check for foreignId()->constrained() pattern
        let constrained_regex = Regex::new(
            r#"\$table\s*->\s*foreignId\s*\(\s*['"](\w+)['"]\s*\)\s*->\s*constrained\s*\(\s*(?:['"](\w+)['"])?"#
        ).unwrap();

        for caps in constrained_regex.captures_iter(content) {
            let column = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let table = caps.get(2).map(|m| m.as_str().to_string());

            // Infer table name from column (user_id -> users)
            let inferred_table = table.unwrap_or_else(|| {
                column.strip_suffix("_id")
                    .map(|s| format!("{}s", s))
                    .unwrap_or_default()
            });

            let match_start = caps.get(0).map(|m| m.start()).unwrap_or(0);
            let context = self.get_line_context(content, match_start);

            let on_delete = self.extract_on_action(&context, "onDelete")
                .or_else(|| self.extract_on_action(&context, "cascadeOnDelete").map(|_| "cascade".to_string()));
            let on_update = self.extract_on_action(&context, "onUpdate")
                .or_else(|| self.extract_on_action(&context, "cascadeOnUpdate").map(|_| "cascade".to_string()));

            foreign_keys.push(serde_json::json!({
                "column": column,
                "references": "id",
                "on_table": inferred_table,
                "on_delete": on_delete,
                "on_update": on_update,
                "constrained": true
            }));
        }

        foreign_keys
    }

    fn extract_on_action(&self, context: &str, action: &str) -> Option<String> {
        let pattern = format!(r#"->\s*{}\s*\(\s*['"](\w+)['"]"#, action);
        let regex = Regex::new(&pattern).ok()?;

        regex.captures(context)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn extract_indexes(&self, content: &str) -> Vec<serde_json::Value> {
        let mut indexes = Vec::new();

        // Primary keys
        for caps in self.primary_key_regex.captures_iter(content) {
            let columns = self.extract_index_columns(&caps);
            indexes.push(serde_json::json!({
                "type": "primary",
                "columns": columns
            }));
        }

        // Unique indexes
        for caps in self.unique_regex.captures_iter(content) {
            let columns = self.extract_index_columns(&caps);
            indexes.push(serde_json::json!({
                "type": "unique",
                "columns": columns
            }));
        }

        // Regular indexes
        for caps in self.index_regex.captures_iter(content) {
            let columns = self.extract_index_columns(&caps);
            indexes.push(serde_json::json!({
                "type": "index",
                "columns": columns
            }));
        }

        indexes
    }

    fn extract_index_columns(&self, caps: &regex::Captures) -> Vec<String> {
        // Single column
        if let Some(single) = caps.get(1) {
            return vec![single.as_str().to_string()];
        }

        // Array of columns
        if let Some(array) = caps.get(2) {
            return array.as_str()
                .split(',')
                .map(|s| s.trim().trim_matches(|c| c == '\'' || c == '"').to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        Vec::new()
    }

    fn extract_use_statements(&self, content: &str, parsed: &mut ParsedFile) {
        let use_regex = Regex::new(r"(?m)^\s*use\s+([\w\\]+)(?:\s+as\s+(\w+))?\s*;").unwrap();

        for caps in use_regex.captures_iter(content) {
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
}

impl Default for MigrationParser {
    fn default() -> Self {
        Self::new()
    }
}
