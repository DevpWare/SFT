use regex::Regex;
use std::fs;

use crate::models::{Dependency, ParsedFile, SourceFile, Symbol, SymbolType};
use crate::parsers::{ParseError, ParserConfig, ParserResult};

/// Represents a parsed Laravel route
#[derive(Debug, Clone, serde::Serialize)]
pub struct RouteDefinition {
    pub method: String,
    pub uri: String,
    pub action: RouteAction,
    pub name: Option<String>,
    pub middleware: Vec<String>,
    pub prefix: Option<String>,
}

/// Route action - controller method or closure
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum RouteAction {
    Controller { controller: String, method: String },
    Closure,
    View { view: String },
    Redirect { to: String },
}

/// Parser for Laravel route files (web.php, api.php, etc.)
pub struct RouteParser {
    // Basic route patterns
    route_regex: Regex,
    // Resource route pattern
    resource_regex: Regex,
    // API resource route pattern
    api_resource_regex: Regex,
    // Route group pattern
    group_regex: Regex,
    // Controller action pattern [Controller::class, 'method']
    controller_action_regex: Regex,
    // String controller pattern 'Controller@method'
    string_controller_regex: Regex,
    // Route name pattern ->name('xxx')
    route_name_regex: Regex,
    // Middleware pattern ->middleware(['xxx'])
    middleware_regex: Regex,
    // Prefix pattern ->prefix('xxx')
    prefix_regex: Regex,
    // View route
    view_route_regex: Regex,
    // Redirect route
    redirect_route_regex: Regex,
}

impl RouteParser {
    pub fn new() -> Self {
        Self {
            // Match: Route::get('/path', ...)
            route_regex: Regex::new(
                r#"Route::(get|post|put|patch|delete|options|any|match)\s*\(\s*['"]([^'"]+)['"]"#,
            )
            .unwrap(),

            // Match: Route::resource('photos', PhotoController::class)
            resource_regex: Regex::new(
                r#"Route::resource\s*\(\s*['"]([^'"]+)['"]\s*,\s*([^)]+)\)"#,
            )
            .unwrap(),

            // Match: Route::apiResource('photos', PhotoController::class)
            api_resource_regex: Regex::new(
                r#"Route::apiResource\s*\(\s*['"]([^'"]+)['"]\s*,\s*([^)]+)\)"#,
            )
            .unwrap(),

            // Match: Route::group(['prefix' => 'admin', ...], function() { ... })
            group_regex: Regex::new(r"Route::group\s*\(\s*\[([^\]]*)\]").unwrap(),

            // Match: [UserController::class, 'index']
            controller_action_regex: Regex::new(
                r#"\[\s*([A-Z]\w+)::class\s*,\s*['"](\w+)['"]\s*\]"#,
            )
            .unwrap(),

            // Match: 'UserController@index' (legacy style)
            string_controller_regex: Regex::new(r#"['"]([A-Z]\w+)@(\w+)['"]"#).unwrap(),

            // Match: ->name('users.index')
            route_name_regex: Regex::new(r#"->\s*name\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),

            // Match: ->middleware(['auth', 'admin']) or ->middleware('auth')
            middleware_regex: Regex::new(
                r#"->\s*middleware\s*\(\s*(?:\[([^\]]+)\]|['"]([^'"]+)['"])"#,
            )
            .unwrap(),

            // Match: ->prefix('admin')
            prefix_regex: Regex::new(r#"->\s*prefix\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),

            // Match: Route::view('/welcome', 'welcome')
            view_route_regex: Regex::new(
                r#"Route::view\s*\(\s*['"]([^'"]+)['"]\s*,\s*['"]([^'"]+)['"]"#,
            )
            .unwrap(),

            // Match: Route::redirect('/here', '/there')
            redirect_route_regex: Regex::new(
                r#"Route::redirect\s*\(\s*['"]([^'"]+)['"]\s*,\s*['"]([^'"]+)['"]"#,
            )
            .unwrap(),
        }
    }

    pub async fn parse(
        &self,
        file: &SourceFile,
        _config: &ParserConfig,
    ) -> ParserResult<ParsedFile> {
        let content = fs::read_to_string(&file.absolute_path).map_err(ParseError::Io)?;

        let mut parsed = ParsedFile::new(file.clone());

        // Determine route file type (web, api, channels, console)
        let route_type = self.detect_route_type(&file.name);
        parsed.metadata.insert(
            "route_type".to_string(),
            serde_json::Value::String(route_type.clone()),
        );

        // Extract use statements for controller references
        self.extract_use_statements(&content, &mut parsed);

        // Extract all routes
        let routes = self.extract_routes(&content);
        if !routes.is_empty() {
            parsed
                .metadata
                .insert("routes".to_string(), serde_json::json!(routes));

            parsed
                .metadata
                .insert("route_count".to_string(), serde_json::json!(routes.len()));
        }

        // Extract resource routes
        let resources = self.extract_resource_routes(&content);
        if !resources.is_empty() {
            parsed
                .metadata
                .insert("resource_routes".to_string(), serde_json::json!(resources));
        }

        // Extract API resource routes
        let api_resources = self.extract_api_resource_routes(&content);
        if !api_resources.is_empty() {
            parsed.metadata.insert(
                "api_resource_routes".to_string(),
                serde_json::json!(api_resources),
            );
        }

        // Extract route groups
        let groups = self.extract_route_groups(&content);
        if !groups.is_empty() {
            parsed
                .metadata
                .insert("route_groups".to_string(), serde_json::json!(groups));
        }

        // Extract all referenced controllers
        let controllers = self.extract_referenced_controllers(&content);
        if !controllers.is_empty() {
            parsed.metadata.insert(
                "controllers_referenced".to_string(),
                serde_json::json!(controllers),
            );
        }

        // Extract all middleware used
        let middlewares = self.extract_all_middleware(&content);
        if !middlewares.is_empty() {
            parsed.metadata.insert(
                "middlewares_used".to_string(),
                serde_json::json!(middlewares),
            );
        }

        // Add route file as a symbol
        parsed.add_symbol(Symbol {
            name: file.name.clone(),
            qualified_name: format!("routes/{}", file.name),
            symbol_type: SymbolType::Unit,
            visibility: Some("public".to_string()),
            is_abstract: None,
            is_static: None,
            extends: None,
            implements: None,
            line_start: None,
            line_end: None,
        });

        Ok(parsed)
    }

    fn detect_route_type(&self, filename: &str) -> String {
        match filename {
            "web.php" => "web".to_string(),
            "api.php" => "api".to_string(),
            "channels.php" => "channels".to_string(),
            "console.php" => "console".to_string(),
            _ => "custom".to_string(),
        }
    }

    fn extract_use_statements(&self, content: &str, parsed: &mut ParsedFile) {
        let use_regex = Regex::new(r"(?m)^\s*use\s+([\w\\]+)(?:\s+as\s+(\w+))?\s*;").unwrap();

        for caps in use_regex.captures_iter(content) {
            let target = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
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

    fn extract_routes(&self, content: &str) -> Vec<serde_json::Value> {
        let mut routes = Vec::new();

        // Find basic routes (get, post, put, etc.)
        for caps in self.route_regex.captures_iter(content) {
            let method = caps
                .get(1)
                .map(|m| m.as_str().to_uppercase())
                .unwrap_or_default();
            let uri = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            // Get the rest of the line/statement for additional info
            let match_start = caps.get(0).map(|m| m.start()).unwrap_or(0);
            let context = self.get_route_context(content, match_start);

            let action = self.extract_route_action(&context);
            let name = self.extract_route_name(&context);
            let middleware = self.extract_middleware(&context);

            routes.push(serde_json::json!({
                "method": method,
                "uri": uri,
                "action": action,
                "name": name,
                "middleware": middleware
            }));
        }

        // Find view routes
        for caps in self.view_route_regex.captures_iter(content) {
            let uri = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let view = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            routes.push(serde_json::json!({
                "method": "GET",
                "uri": uri,
                "action": {
                    "type": "View",
                    "view": view
                },
                "name": null,
                "middleware": []
            }));
        }

        // Find redirect routes
        for caps in self.redirect_route_regex.captures_iter(content) {
            let from = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let to = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            routes.push(serde_json::json!({
                "method": "GET",
                "uri": from,
                "action": {
                    "type": "Redirect",
                    "to": to
                },
                "name": null,
                "middleware": []
            }));
        }

        routes
    }

    fn get_route_context(&self, content: &str, start: usize) -> String {
        // Get approximately the next 500 characters or until semicolon/newline pattern
        let remaining = &content[start..];
        let end = remaining
            .find(';')
            .map(|i| i + 1)
            .unwrap_or_else(|| remaining.len().min(500));

        remaining[..end].to_string()
    }

    fn extract_route_action(&self, context: &str) -> serde_json::Value {
        // Try controller class syntax first: [Controller::class, 'method']
        if let Some(caps) = self.controller_action_regex.captures(context) {
            let controller = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let method = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            return serde_json::json!({
                "type": "Controller",
                "controller": controller,
                "method": method
            });
        }

        // Try legacy string syntax: 'Controller@method'
        if let Some(caps) = self.string_controller_regex.captures(context) {
            let controller = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let method = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            return serde_json::json!({
                "type": "Controller",
                "controller": controller,
                "method": method
            });
        }

        // Check for closure
        if context.contains("function") && context.contains('{') {
            return serde_json::json!({
                "type": "Closure"
            });
        }

        serde_json::json!({
            "type": "Unknown"
        })
    }

    fn extract_route_name(&self, context: &str) -> Option<String> {
        self.route_name_regex
            .captures(context)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn extract_middleware(&self, context: &str) -> Vec<String> {
        let mut middlewares = Vec::new();

        if let Some(caps) = self.middleware_regex.captures(context) {
            // Array format: ['auth', 'admin']
            if let Some(array_list) = caps.get(1) {
                for item in array_list.as_str().split(',') {
                    let cleaned = item.trim().trim_matches(|c| c == '\'' || c == '"');
                    if !cleaned.is_empty() && !middlewares.contains(&cleaned.to_string()) {
                        middlewares.push(cleaned.to_string());
                    }
                }
            }
            // Single string format: 'auth'
            else if let Some(single) = caps.get(2) {
                let cleaned = single.as_str().trim();
                if !cleaned.is_empty() {
                    middlewares.push(cleaned.to_string());
                }
            }
        }

        middlewares
    }

    fn extract_resource_routes(&self, content: &str) -> Vec<serde_json::Value> {
        let mut resources = Vec::new();

        for caps in self.resource_regex.captures_iter(content) {
            let resource_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let controller = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");

            // Extract controller name from Controller::class
            let controller_name = if controller.contains("::class") {
                controller.replace("::class", "").trim().to_string()
            } else {
                controller
                    .trim_matches(|c| c == '\'' || c == '"')
                    .to_string()
            };

            resources.push(serde_json::json!({
                "name": resource_name,
                "controller": controller_name,
                "type": "resource",
                "routes": ["index", "create", "store", "show", "edit", "update", "destroy"]
            }));
        }

        resources
    }

    fn extract_api_resource_routes(&self, content: &str) -> Vec<serde_json::Value> {
        let mut resources = Vec::new();

        for caps in self.api_resource_regex.captures_iter(content) {
            let resource_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let controller = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");

            let controller_name = if controller.contains("::class") {
                controller.replace("::class", "").trim().to_string()
            } else {
                controller
                    .trim_matches(|c| c == '\'' || c == '"')
                    .to_string()
            };

            resources.push(serde_json::json!({
                "name": resource_name,
                "controller": controller_name,
                "type": "apiResource",
                "routes": ["index", "store", "show", "update", "destroy"]
            }));
        }

        resources
    }

    fn extract_route_groups(&self, content: &str) -> Vec<serde_json::Value> {
        let mut groups = Vec::new();

        for caps in self.group_regex.captures_iter(content) {
            if let Some(options) = caps.get(1) {
                let options_str = options.as_str();

                let prefix = self.extract_group_option(options_str, "prefix");
                let middleware = self.extract_group_middleware(options_str);
                let namespace = self.extract_group_option(options_str, "namespace");
                let name_prefix = self.extract_group_option(options_str, "as");

                groups.push(serde_json::json!({
                    "prefix": prefix,
                    "middleware": middleware,
                    "namespace": namespace,
                    "name_prefix": name_prefix
                }));
            }
        }

        groups
    }

    fn extract_group_option(&self, options: &str, key: &str) -> Option<String> {
        let pattern = format!(r#"['"]{}['"]\s*=>\s*['"]([^'"]+)['"]"#, key);
        let regex = Regex::new(&pattern).ok()?;

        regex
            .captures(options)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn extract_group_middleware(&self, options: &str) -> Vec<String> {
        let mut middlewares = Vec::new();

        // Try array format: 'middleware' => ['auth', 'admin']
        let array_regex = Regex::new(r#"['"]middleware['"]\s*=>\s*\[([^\]]+)\]"#).unwrap();
        if let Some(caps) = array_regex.captures(options) {
            if let Some(list) = caps.get(1) {
                for item in list.as_str().split(',') {
                    let cleaned = item.trim().trim_matches(|c| c == '\'' || c == '"');
                    if !cleaned.is_empty() {
                        middlewares.push(cleaned.to_string());
                    }
                }
                return middlewares;
            }
        }

        // Try single value: 'middleware' => 'auth'
        let single_regex = Regex::new(r#"['"]middleware['"]\s*=>\s*['"]([^'"]+)['"]"#).unwrap();
        if let Some(caps) = single_regex.captures(options) {
            if let Some(mw) = caps.get(1) {
                middlewares.push(mw.as_str().to_string());
            }
        }

        middlewares
    }

    fn extract_referenced_controllers(&self, content: &str) -> Vec<String> {
        let mut controllers = Vec::new();

        // From controller class syntax
        for caps in self.controller_action_regex.captures_iter(content) {
            if let Some(controller) = caps.get(1) {
                let name = controller.as_str().to_string();
                if !controllers.contains(&name) {
                    controllers.push(name);
                }
            }
        }

        // From string syntax
        for caps in self.string_controller_regex.captures_iter(content) {
            if let Some(controller) = caps.get(1) {
                let name = controller.as_str().to_string();
                if !controllers.contains(&name) {
                    controllers.push(name);
                }
            }
        }

        // From resource routes
        let resource_controller_regex = Regex::new(r"([A-Z]\w+)::class").unwrap();
        for caps in resource_controller_regex.captures_iter(content) {
            if let Some(controller) = caps.get(1) {
                let name = controller.as_str().to_string();
                if !controllers.contains(&name) && name.ends_with("Controller") {
                    controllers.push(name);
                }
            }
        }

        controllers
    }

    fn extract_all_middleware(&self, content: &str) -> Vec<String> {
        let mut middlewares = Vec::new();

        // From ->middleware() calls
        for caps in self.middleware_regex.captures_iter(content) {
            if let Some(array_list) = caps.get(1) {
                for item in array_list.as_str().split(',') {
                    let cleaned = item.trim().trim_matches(|c| c == '\'' || c == '"');
                    if !cleaned.is_empty() && !middlewares.contains(&cleaned.to_string()) {
                        middlewares.push(cleaned.to_string());
                    }
                }
            } else if let Some(single) = caps.get(2) {
                let cleaned = single.as_str().trim();
                if !cleaned.is_empty() && !middlewares.contains(&cleaned.to_string()) {
                    middlewares.push(cleaned.to_string());
                }
            }
        }

        // From group options
        let group_mw_regex =
            Regex::new(r#"['"]middleware['"]\s*=>\s*(?:\[([^\]]+)\]|['"]([^'"]+)['"])"#).unwrap();
        for caps in group_mw_regex.captures_iter(content) {
            if let Some(list) = caps.get(1) {
                for item in list.as_str().split(',') {
                    let cleaned = item.trim().trim_matches(|c| c == '\'' || c == '"');
                    if !cleaned.is_empty() && !middlewares.contains(&cleaned.to_string()) {
                        middlewares.push(cleaned.to_string());
                    }
                }
            } else if let Some(single) = caps.get(2) {
                let name = single.as_str().to_string();
                if !middlewares.contains(&name) {
                    middlewares.push(name);
                }
            }
        }

        middlewares
    }
}

impl Default for RouteParser {
    fn default() -> Self {
        Self::new()
    }
}
