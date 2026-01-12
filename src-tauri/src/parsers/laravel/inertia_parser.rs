use regex::Regex;
use std::fs;

use crate::models::{Dependency, ParsedFile, SourceFile, Symbol, SymbolType};
use crate::parsers::{ParseError, ParserConfig, ParserResult};

/// Parser for Inertia.js page components (Vue, React, Svelte)
pub struct InertiaParser {
    // Vue imports
    vue_import_regex: Regex,
    vue_component_regex: Regex,
    vue_props_regex: Regex,
    vue_emit_regex: Regex,

    // React imports
    react_import_regex: Regex,
    react_component_regex: Regex,

    // Inertia specific
    inertia_link_regex: Regex,
    inertia_form_regex: Regex,
    inertia_router_regex: Regex,
    use_page_regex: Regex,
    use_form_regex: Regex,

    // TypeScript props
    define_props_regex: Regex,
    interface_regex: Regex,
    type_regex: Regex,
}

impl InertiaParser {
    pub fn new() -> Self {
        Self {
            // Match: import Component from './Component.vue'
            vue_import_regex: Regex::new(
                r#"import\s+(\w+)\s+from\s+['"]([^'"]+\.vue)['"]"#
            ).unwrap(),

            // Match: <ComponentName /> or <ComponentName>
            vue_component_regex: Regex::new(
                r"<([A-Z][a-zA-Z0-9]+)[\s/>]"
            ).unwrap(),

            // Match: defineProps<{...}>() or defineProps({...})
            vue_props_regex: Regex::new(
                r"defineProps\s*(?:<([^>]+)>)?\s*\("
            ).unwrap(),

            // Match: defineEmits(['event1', 'event2'])
            vue_emit_regex: Regex::new(
                r#"defineEmits\s*\(\s*\[([^\]]+)\]"#
            ).unwrap(),

            // Match: import { Component } from 'react' or import Component from './Component'
            react_import_regex: Regex::new(
                r#"import\s+(?:\{([^}]+)\}|(\w+))\s+from\s+['"]([^'"]+)['"]"#
            ).unwrap(),

            // Match: function ComponentName() or const ComponentName = () =>
            react_component_regex: Regex::new(
                r"(?:function|const)\s+([A-Z][a-zA-Z0-9]+)\s*(?:=\s*\([^)]*\)\s*=>|\()"
            ).unwrap(),

            // Match: <Link href="/path"> or <InertiaLink>
            inertia_link_regex: Regex::new(
                r#"<(?:Link|InertiaLink)\s+[^>]*href\s*=\s*['"]([^'"]+)['"]"#
            ).unwrap(),

            // Match: useForm({ ... }) or <form> with @submit
            inertia_form_regex: Regex::new(
                r"useForm\s*\("
            ).unwrap(),

            // Match: router.visit('/path') or router.get('/path')
            inertia_router_regex: Regex::new(
                r#"router\.(visit|get|post|put|patch|delete)\s*\(\s*['"]([^'"]+)['"]"#
            ).unwrap(),

            // Match: usePage()
            use_page_regex: Regex::new(
                r"usePage\s*\(\s*\)"
            ).unwrap(),

            // Match: useForm()
            use_form_regex: Regex::new(
                r"useForm\s*\("
            ).unwrap(),

            // Match: defineProps<Props>() with type extraction
            define_props_regex: Regex::new(
                r"defineProps\s*<\s*(\w+)\s*>"
            ).unwrap(),

            // Match: interface Props { ... }
            interface_regex: Regex::new(
                r"interface\s+(\w+)\s*\{"
            ).unwrap(),

            // Match: type Props = { ... }
            type_regex: Regex::new(
                r"type\s+(\w+)\s*="
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

        // Determine page name from path
        let page_name = self.extract_page_name(&file.path);
        parsed.metadata.insert(
            "page_name".to_string(),
            serde_json::Value::String(page_name.clone()),
        );

        // Detect framework (Vue, React, Svelte)
        let framework = self.detect_framework(&content, &file.name);
        parsed.metadata.insert(
            "framework".to_string(),
            serde_json::Value::String(framework.clone()),
        );

        parsed.metadata.insert(
            "is_inertia_page".to_string(),
            serde_json::json!(true),
        );

        // Extract imports
        let imports = self.extract_imports(&content, &framework);
        if !imports.is_empty() {
            parsed.metadata.insert(
                "imports".to_string(),
                serde_json::json!(imports),
            );

            for import in &imports {
                if let Some(path) = import.get("path").and_then(|p| p.as_str()) {
                    parsed.add_dependency(Dependency {
                        target: path.to_string(),
                        alias: import.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()),
                        line_number: None,
                        is_interface: false,
                        is_implementation: false,
                    });
                }
            }
        }

        // Extract child components used
        let components = self.extract_child_components(&content);
        if !components.is_empty() {
            parsed.metadata.insert(
                "child_components".to_string(),
                serde_json::json!(components),
            );
        }

        // Extract Inertia links (routes referenced)
        let links = self.extract_inertia_links(&content);
        if !links.is_empty() {
            parsed.metadata.insert(
                "inertia_links".to_string(),
                serde_json::json!(links),
            );
        }

        // Extract router calls
        let router_calls = self.extract_router_calls(&content);
        if !router_calls.is_empty() {
            parsed.metadata.insert(
                "router_calls".to_string(),
                serde_json::json!(router_calls),
            );
        }

        // Check for Inertia hooks usage
        let uses_page = self.use_page_regex.is_match(&content);
        let uses_form = self.use_form_regex.is_match(&content);

        if uses_page {
            parsed.metadata.insert(
                "uses_page_props".to_string(),
                serde_json::json!(true),
            );
        }

        if uses_form {
            parsed.metadata.insert(
                "uses_inertia_form".to_string(),
                serde_json::json!(true),
            );
        }

        // Extract props (Vue 3 style)
        if framework == "vue" {
            let props = self.extract_vue_props(&content);
            if !props.is_empty() {
                parsed.metadata.insert(
                    "props".to_string(),
                    serde_json::json!(props),
                );
            }

            let emits = self.extract_vue_emits(&content);
            if !emits.is_empty() {
                parsed.metadata.insert(
                    "emits".to_string(),
                    serde_json::json!(emits),
                );
            }
        }

        // Extract TypeScript types/interfaces
        let types = self.extract_typescript_types(&content);
        if !types.is_empty() {
            parsed.metadata.insert(
                "typescript_types".to_string(),
                serde_json::json!(types),
            );
        }

        // Add the page as a symbol
        parsed.add_symbol(Symbol {
            name: page_name.clone(),
            qualified_name: format!("inertia:{}", page_name),
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

    fn extract_page_name(&self, path: &str) -> String {
        // Convert path like "resources/js/Pages/Dashboard.vue" to "Dashboard"
        // or "resources/js/Pages/Users/Index.vue" to "Users/Index"
        path.replace("resources/js/Pages/", "")
            .replace("resources/js/pages/", "")
            .replace(".vue", "")
            .replace(".jsx", "")
            .replace(".tsx", "")
            .replace(".svelte", "")
    }

    fn detect_framework(&self, content: &str, filename: &str) -> String {
        if filename.ends_with(".vue") {
            return "vue".to_string();
        }
        if filename.ends_with(".svelte") {
            return "svelte".to_string();
        }
        if filename.ends_with(".jsx") || filename.ends_with(".tsx") {
            return "react".to_string();
        }

        // Try to detect from content
        if content.contains("<template>") || content.contains("<script setup") {
            return "vue".to_string();
        }
        if content.contains("import React") || content.contains("from 'react'") {
            return "react".to_string();
        }

        "unknown".to_string()
    }

    fn extract_imports(&self, content: &str, framework: &str) -> Vec<serde_json::Value> {
        let mut imports = Vec::new();

        if framework == "vue" {
            for caps in self.vue_import_regex.captures_iter(content) {
                let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let path = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                imports.push(serde_json::json!({
                    "name": name,
                    "path": path,
                    "type": "vue_component"
                }));
            }
        }

        // General ES6 imports
        let general_import = Regex::new(
            r#"import\s+(?:\{([^}]+)\}|(\w+))\s+from\s+['"]([^'"]+)['"]"#
        ).unwrap();

        for caps in general_import.captures_iter(content) {
            let named = caps.get(1).map(|m| m.as_str());
            let default = caps.get(2).map(|m| m.as_str());
            let path = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            if let Some(names) = named {
                for name in names.split(',') {
                    let clean_name = name.trim().split(" as ").next().unwrap_or("").trim();
                    if !clean_name.is_empty() {
                        imports.push(serde_json::json!({
                            "name": clean_name,
                            "path": path,
                            "type": "named"
                        }));
                    }
                }
            }

            if let Some(name) = default {
                if !imports.iter().any(|i| i.get("name") == Some(&serde_json::json!(name))) {
                    imports.push(serde_json::json!({
                        "name": name,
                        "path": path,
                        "type": "default"
                    }));
                }
            }
        }

        imports
    }

    fn extract_child_components(&self, content: &str) -> Vec<String> {
        let mut components = Vec::new();

        for caps in self.vue_component_regex.captures_iter(content) {
            if let Some(name) = caps.get(1) {
                let component = name.as_str().to_string();
                // Filter out HTML elements and common non-components
                if !["Head", "Link", "InertiaLink", "Teleport", "Transition", "TransitionGroup",
                     "KeepAlive", "Suspense", "Fragment"].contains(&component.as_str())
                    && !components.contains(&component)
                {
                    components.push(component);
                }
            }
        }

        components
    }

    fn extract_inertia_links(&self, content: &str) -> Vec<String> {
        let mut links = Vec::new();

        for caps in self.inertia_link_regex.captures_iter(content) {
            if let Some(href) = caps.get(1) {
                let link = href.as_str().to_string();
                if !links.contains(&link) {
                    links.push(link);
                }
            }
        }

        links
    }

    fn extract_router_calls(&self, content: &str) -> Vec<serde_json::Value> {
        let mut calls = Vec::new();

        for caps in self.inertia_router_regex.captures_iter(content) {
            let method = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let path = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            calls.push(serde_json::json!({
                "method": method,
                "path": path
            }));
        }

        calls
    }

    fn extract_vue_props(&self, content: &str) -> Vec<String> {
        let mut props = Vec::new();

        // Try to extract from defineProps
        let props_object_regex = Regex::new(
            r#"defineProps\s*\(\s*\{([^}]+)\}"#
        ).ok();

        if let Some(regex) = props_object_regex {
            for caps in regex.captures_iter(content) {
                if let Some(props_str) = caps.get(1) {
                    let prop_name_regex = Regex::new(r"(\w+)\s*:").unwrap();
                    for prop_caps in prop_name_regex.captures_iter(props_str.as_str()) {
                        if let Some(prop) = prop_caps.get(1) {
                            let name = prop.as_str().to_string();
                            if !props.contains(&name) {
                                props.push(name);
                            }
                        }
                    }
                }
            }
        }

        props
    }

    fn extract_vue_emits(&self, content: &str) -> Vec<String> {
        let mut emits = Vec::new();

        for caps in self.vue_emit_regex.captures_iter(content) {
            if let Some(emits_str) = caps.get(1) {
                for item in emits_str.as_str().split(',') {
                    let cleaned = item.trim().trim_matches(|c| c == '\'' || c == '"');
                    if !cleaned.is_empty() && !emits.contains(&cleaned.to_string()) {
                        emits.push(cleaned.to_string());
                    }
                }
            }
        }

        emits
    }

    fn extract_typescript_types(&self, content: &str) -> Vec<String> {
        let mut types = Vec::new();

        for caps in self.interface_regex.captures_iter(content) {
            if let Some(name) = caps.get(1) {
                let type_name = name.as_str().to_string();
                if !types.contains(&type_name) {
                    types.push(type_name);
                }
            }
        }

        for caps in self.type_regex.captures_iter(content) {
            if let Some(name) = caps.get(1) {
                let type_name = name.as_str().to_string();
                if !types.contains(&type_name) {
                    types.push(type_name);
                }
            }
        }

        types
    }
}

impl Default for InertiaParser {
    fn default() -> Self {
        Self::new()
    }
}
