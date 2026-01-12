use regex::Regex;
use std::fs;

use crate::models::{Dependency, ParsedFile, SourceFile, Symbol, SymbolType};
use crate::parsers::{ParseError, ParserConfig, ParserResult};

/// Parser for Laravel Controllers
pub struct ControllerParser {
    namespace_regex: Regex,
    use_regex: Regex,
    class_regex: Regex,
    method_regex: Regex,
    route_method_regex: Regex,
    middleware_regex: Regex,
    resource_methods: Vec<&'static str>,
    // Inertia support
    inertia_render_regex: Regex,
    inertia_function_regex: Regex,
}

impl ControllerParser {
    pub fn new() -> Self {
        Self {
            namespace_regex: Regex::new(r"(?m)^\s*namespace\s+([\w\\]+)\s*;").unwrap(),
            use_regex: Regex::new(r"(?m)^\s*use\s+([\w\\]+)(?:\s+as\s+(\w+))?\s*;").unwrap(),
            class_regex: Regex::new(
                r"(?m)^\s*class\s+(\w+)(?:\s+extends\s+([\w\\]+))?"
            ).unwrap(),
            method_regex: Regex::new(
                r"(?m)^\s*(public|protected|private)\s+function\s+(\w+)\s*\(([^)]*)\)"
            ).unwrap(),
            // Detect route-related annotations or method calls
            route_method_regex: Regex::new(
                r#"(?m)Route::(get|post|put|patch|delete|options|any)\s*\(\s*['"]([^'"]+)['"]"#
            ).unwrap(),
            // Match: $this->middleware('auth')
            middleware_regex: Regex::new(
                r#"\$this\s*->\s*middleware\s*\(\s*['"]([^'"]+)['"]"#
            ).unwrap(),
            resource_methods: vec![
                "index", "create", "store", "show", "edit", "update", "destroy"
            ],
            // Match: Inertia::render('Pages/Dashboard') or Inertia::render('Dashboard')
            inertia_render_regex: Regex::new(
                r#"Inertia::render\s*\(\s*['"]([^'"]+)['"]"#
            ).unwrap(),
            // Match: inertia('Pages/Dashboard') or return inertia('Dashboard', [...])
            inertia_function_regex: Regex::new(
                r#"(?:return\s+)?inertia\s*\(\s*['"]([^'"]+)['"]"#
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

        // Extract controller class
        self.extract_controller_class(&content, &namespace, &mut parsed);

        // Extract controller methods (actions)
        self.extract_controller_methods(&content, &mut parsed);

        // Extract middleware usage
        let middlewares = self.extract_middlewares(&content);
        if !middlewares.is_empty() {
            parsed.metadata.insert(
                "middlewares".to_string(),
                serde_json::json!(middlewares),
            );
        }

        // Detect if it's a resource controller
        let is_resource = self.is_resource_controller(&content);
        parsed.metadata.insert(
            "is_resource_controller".to_string(),
            serde_json::Value::Bool(is_resource),
        );

        // Extract view references
        let views = self.extract_view_references(&content);
        if !views.is_empty() {
            parsed.metadata.insert(
                "views_referenced".to_string(),
                serde_json::json!(views),
            );
        }

        // Extract model references
        let models = self.extract_model_references(&content);
        if !models.is_empty() {
            parsed.metadata.insert(
                "models_referenced".to_string(),
                serde_json::json!(models),
            );
        }

        // Extract Inertia page references
        let inertia_pages = self.extract_inertia_pages(&content);
        if !inertia_pages.is_empty() {
            parsed.metadata.insert(
                "inertia_pages".to_string(),
                serde_json::json!(inertia_pages),
            );
            parsed.metadata.insert(
                "uses_inertia".to_string(),
                serde_json::json!(true),
            );
        }

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

    fn extract_controller_class(
        &self,
        content: &str,
        namespace: &Option<String>,
        parsed: &mut ParsedFile,
    ) {
        if let Some(caps) = self.class_regex.captures(content) {
            let class_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let extends = caps.get(2).map(|m| m.as_str().to_string());

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
                    implements: None,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }

    fn extract_controller_methods(&self, content: &str, parsed: &mut ParsedFile) {
        for caps in self.method_regex.captures_iter(content) {
            let visibility = caps.get(1).map(|m| m.as_str().to_string());
            let method_name = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            let _params = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();

            // Skip constructor and other magic methods for action listing
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
                    is_static: None,
                    extends: None,
                    implements: None,
                    line_start: None,
                    line_end: None,
                });
            }
        }
    }

    fn extract_middlewares(&self, content: &str) -> Vec<String> {
        let mut middlewares = Vec::new();

        for caps in self.middleware_regex.captures_iter(content) {
            if let Some(m) = caps.get(1) {
                let middleware = m.as_str().to_string();
                if !middlewares.contains(&middleware) {
                    middlewares.push(middleware);
                }
            }
        }

        // Also check for middleware in constructor patterns
        let group_middleware = Regex::new(
            r"\$this\s*->\s*middleware\s*\(\s*\[([^\]]+)\]"
        ).unwrap();

        for caps in group_middleware.captures_iter(content) {
            if let Some(list) = caps.get(1) {
                for item in list.as_str().split(',') {
                    let cleaned = item.trim().trim_matches(|c| c == '\'' || c == '"');
                    if !cleaned.is_empty() && !middlewares.contains(&cleaned.to_string()) {
                        middlewares.push(cleaned.to_string());
                    }
                }
            }
        }

        middlewares
    }

    fn is_resource_controller(&self, content: &str) -> bool {
        let mut found_methods = 0;

        for method in &self.resource_methods {
            let pattern = format!(r"function\s+{}\s*\(", method);
            if Regex::new(&pattern).unwrap().is_match(content) {
                found_methods += 1;
            }
        }

        // Consider it a resource controller if it has at least 4 resource methods
        found_methods >= 4
    }

    fn extract_view_references(&self, content: &str) -> Vec<String> {
        let mut views = Vec::new();

        // Match: view('users.index') or View::make('users.show')
        let view_regex = Regex::new(r#"(?:view|View::make)\s*\(\s*['"]([^'"]+)['"]"#).unwrap();

        for caps in view_regex.captures_iter(content) {
            if let Some(v) = caps.get(1) {
                let view = v.as_str().to_string();
                if !views.contains(&view) {
                    views.push(view);
                }
            }
        }

        views
    }

    fn extract_model_references(&self, content: &str) -> Vec<String> {
        let mut models = Vec::new();

        // Match Model::query() or Model::find() or Model::where() etc.
        let model_static_regex = Regex::new(
            r"([A-Z][a-zA-Z]+)::(find|findOrFail|where|all|create|firstOrCreate|updateOrCreate|query|with)\s*\("
        ).unwrap();

        for caps in model_static_regex.captures_iter(content) {
            if let Some(m) = caps.get(1) {
                let model = m.as_str().to_string();
                // Exclude common non-model classes
                if !["DB", "Auth", "Cache", "Log", "Route", "View", "Request", "Response", "Session", "Config", "App", "Event"]
                    .contains(&model.as_str())
                    && !models.contains(&model)
                {
                    models.push(model);
                }
            }
        }

        // Match type-hinted model parameters: function show(User $user)
        let type_hint_regex = Regex::new(r"function\s+\w+\s*\([^)]*?([A-Z][a-zA-Z]+)\s+\$\w+").unwrap();

        for caps in type_hint_regex.captures_iter(content) {
            if let Some(m) = caps.get(1) {
                let model = m.as_str().to_string();
                if !["Request", "Response", "Collection", "Builder", "Carbon", "Closure", "Exception"]
                    .contains(&model.as_str())
                    && !models.contains(&model)
                {
                    models.push(model);
                }
            }
        }

        models
    }

    fn extract_inertia_pages(&self, content: &str) -> Vec<String> {
        let mut pages = Vec::new();

        // Match Inertia::render('PageName')
        for caps in self.inertia_render_regex.captures_iter(content) {
            if let Some(page) = caps.get(1) {
                let page_name = page.as_str().to_string();
                if !pages.contains(&page_name) {
                    pages.push(page_name);
                }
            }
        }

        // Match inertia('PageName')
        for caps in self.inertia_function_regex.captures_iter(content) {
            if let Some(page) = caps.get(1) {
                let page_name = page.as_str().to_string();
                if !pages.contains(&page_name) {
                    pages.push(page_name);
                }
            }
        }

        pages
    }
}

impl Default for ControllerParser {
    fn default() -> Self {
        Self::new()
    }
}
