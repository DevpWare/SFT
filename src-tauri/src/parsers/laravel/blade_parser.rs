use regex::Regex;
use std::fs;

use crate::models::{Dependency, ParsedFile, SourceFile, Symbol, SymbolType};
use crate::parsers::{ParseError, ParserConfig, ParserResult};

/// Parser for Laravel Blade template files
pub struct BladeParser {
    // Layout inheritance
    extends_regex: Regex,
    section_regex: Regex,
    yield_regex: Regex,

    // Include directives
    include_regex: Regex,
    include_if_regex: Regex,
    include_when_regex: Regex,
    include_first_regex: Regex,
    each_regex: Regex,

    // Components
    component_class_regex: Regex,
    component_x_regex: Regex,
    component_anonymous_regex: Regex,
    slot_regex: Regex,

    // Control structures
    if_regex: Regex,
    foreach_regex: Regex,
    for_regex: Regex,
    while_regex: Regex,
    forelse_regex: Regex,
    switch_regex: Regex,

    // Auth directives
    auth_regex: Regex,
    guest_regex: Regex,
    can_regex: Regex,
    cannot_regex: Regex,

    // Stack directives
    stack_regex: Regex,
    push_regex: Regex,
    prepend_regex: Regex,

    // Variables and output
    echo_regex: Regex,
    raw_echo_regex: Regex,
    php_regex: Regex,

    // Props
    props_regex: Regex,

    // Livewire
    livewire_regex: Regex,

    // Common directives
    csrf_regex: Regex,
    method_regex: Regex,
    error_regex: Regex,
}

impl BladeParser {
    pub fn new() -> Self {
        Self {
            // Match: @extends('layouts.app')
            extends_regex: Regex::new(
                r#"@extends\s*\(\s*['"]([^'"]+)['"]"#
            ).unwrap(),

            // Match: @section('content')
            section_regex: Regex::new(
                r#"@section\s*\(\s*['"]([^'"]+)['"]"#
            ).unwrap(),

            // Match: @yield('content')
            yield_regex: Regex::new(
                r#"@yield\s*\(\s*['"]([^'"]+)['"]"#
            ).unwrap(),

            // Match: @include('partials.header')
            include_regex: Regex::new(
                r#"@include\s*\(\s*['"]([^'"]+)['"]"#
            ).unwrap(),

            // Match: @includeIf('partials.optional')
            include_if_regex: Regex::new(
                r#"@includeIf\s*\(\s*['"]([^'"]+)['"]"#
            ).unwrap(),

            // Match: @includeWhen($condition, 'view')
            include_when_regex: Regex::new(
                r#"@includeWhen\s*\([^,]+,\s*['"]([^'"]+)['"]"#
            ).unwrap(),

            // Match: @includeFirst(['custom', 'default'])
            include_first_regex: Regex::new(
                r"@includeFirst\s*\(\s*\[([^\]]+)\]"
            ).unwrap(),

            // Match: @each('view.name', $items, 'item')
            each_regex: Regex::new(
                r#"@each\s*\(\s*['"]([^'"]+)['"]"#
            ).unwrap(),

            // Match: @component('components.alert')
            component_class_regex: Regex::new(
                r#"@component\s*\(\s*['"]([^'"]+)['"]"#
            ).unwrap(),

            // Match: <x-alert/> or <x-alert>
            component_x_regex: Regex::new(
                r"<x-([a-z][a-z0-9\-\.]*)"
            ).unwrap(),

            // Match: <x-dynamic-component component="name"/>
            component_anonymous_regex: Regex::new(
                r#"<x-dynamic-component\s+[^>]*component\s*=\s*['"]([^'"]+)['"]"#
            ).unwrap(),

            // Match: @slot('header') or <x-slot name="header">
            slot_regex: Regex::new(
                r#"(?:@slot\s*\(\s*['"]([^'"]+)['"]|<x-slot\s+name\s*=\s*['"]([^'"]+)['"])"#
            ).unwrap(),

            // Control structures
            if_regex: Regex::new(r"@if\s*\(").unwrap(),
            foreach_regex: Regex::new(r"@foreach\s*\(").unwrap(),
            for_regex: Regex::new(r"@for\s*\(").unwrap(),
            while_regex: Regex::new(r"@while\s*\(").unwrap(),
            forelse_regex: Regex::new(r"@forelse\s*\(").unwrap(),
            switch_regex: Regex::new(r"@switch\s*\(").unwrap(),

            // Auth directives
            auth_regex: Regex::new(r#"@auth(?:\s*\(\s*['"]([^'"]+)['"])?"#).unwrap(),
            guest_regex: Regex::new(r#"@guest(?:\s*\(\s*['"]([^'"]+)['"])?"#).unwrap(),
            can_regex: Regex::new(r#"@can\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),
            cannot_regex: Regex::new(r#"@cannot\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),

            // Stack directives
            stack_regex: Regex::new(r#"@stack\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),
            push_regex: Regex::new(r#"@push\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),
            prepend_regex: Regex::new(r#"@prepend\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),

            // Variable output
            echo_regex: Regex::new(r"\{\{\s*([^}]+)\s*\}\}").unwrap(),
            raw_echo_regex: Regex::new(r"\{!!\s*([^!]+)\s*!!\}").unwrap(),
            php_regex: Regex::new(r"@php\b").unwrap(),

            // Props
            props_regex: Regex::new(r"@props\s*\(\s*\[([^\]]+)\]").unwrap(),

            // Livewire
            livewire_regex: Regex::new(r#"(?:@livewire\s*\(\s*['"]([^'"]+)['"]|<livewire:([a-z][a-z0-9\-\.]*))"#).unwrap(),

            // Common directives
            csrf_regex: Regex::new(r"@csrf\b").unwrap(),
            method_regex: Regex::new(r#"@method\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),
            error_regex: Regex::new(r#"@error\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),
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

        // Determine view name from path
        let view_name = self.extract_view_name(&file.path);
        parsed.metadata.insert(
            "view_name".to_string(),
            serde_json::Value::String(view_name.clone()),
        );

        // Check if this is a layout
        let is_layout = self.yield_regex.is_match(&content);
        parsed.metadata.insert(
            "is_layout".to_string(),
            serde_json::json!(is_layout),
        );

        // Check if this is a component
        let is_component = file.path.contains("/components/") || self.props_regex.is_match(&content);
        parsed.metadata.insert(
            "is_component".to_string(),
            serde_json::json!(is_component),
        );

        // Extract parent layout
        if let Some(extends) = self.extract_extends(&content) {
            parsed.metadata.insert(
                "extends".to_string(),
                serde_json::Value::String(extends.clone()),
            );

            parsed.add_dependency(Dependency {
                target: format!("view:{}", extends),
                alias: None,
                line_number: None,
                is_interface: false,
                is_implementation: false,
            });
        }

        // Extract sections defined
        let sections = self.extract_sections(&content);
        if !sections.is_empty() {
            parsed.metadata.insert(
                "sections".to_string(),
                serde_json::json!(sections),
            );
        }

        // Extract yields (for layouts)
        let yields = self.extract_yields(&content);
        if !yields.is_empty() {
            parsed.metadata.insert(
                "yields".to_string(),
                serde_json::json!(yields),
            );
        }

        // Extract included views
        let includes = self.extract_includes(&content);
        if !includes.is_empty() {
            parsed.metadata.insert(
                "includes".to_string(),
                serde_json::json!(includes),
            );

            for include in &includes {
                parsed.add_dependency(Dependency {
                    target: format!("view:{}", include),
                    alias: None,
                    line_number: None,
                    is_interface: false,
                    is_implementation: false,
                });
            }
        }

        // Extract Blade components used
        let components = self.extract_components(&content);
        if !components.is_empty() {
            parsed.metadata.insert(
                "components".to_string(),
                serde_json::json!(components),
            );
        }

        // Extract slots defined
        let slots = self.extract_slots(&content);
        if !slots.is_empty() {
            parsed.metadata.insert(
                "slots".to_string(),
                serde_json::json!(slots),
            );
        }

        // Extract stacks
        let stacks = self.extract_stacks(&content);
        if !stacks.is_empty() {
            parsed.metadata.insert(
                "stacks".to_string(),
                serde_json::json!(stacks),
            );
        }

        // Extract pushes
        let pushes = self.extract_pushes(&content);
        if !pushes.is_empty() {
            parsed.metadata.insert(
                "pushes".to_string(),
                serde_json::json!(pushes),
            );
        }

        // Extract props
        let props = self.extract_props(&content);
        if !props.is_empty() {
            parsed.metadata.insert(
                "props".to_string(),
                serde_json::json!(props),
            );
        }

        // Extract Livewire components
        let livewire = self.extract_livewire_components(&content);
        if !livewire.is_empty() {
            parsed.metadata.insert(
                "livewire_components".to_string(),
                serde_json::json!(livewire),
            );
        }

        // Extract permissions/abilities used
        let permissions = self.extract_permissions(&content);
        if !permissions.is_empty() {
            parsed.metadata.insert(
                "permissions".to_string(),
                serde_json::json!(permissions),
            );
        }

        // Extract form errors referenced
        let errors = self.extract_error_bags(&content);
        if !errors.is_empty() {
            parsed.metadata.insert(
                "error_fields".to_string(),
                serde_json::json!(errors),
            );
        }

        // Count directives usage
        let directive_counts = self.count_directives(&content);
        parsed.metadata.insert(
            "directive_counts".to_string(),
            serde_json::json!(directive_counts),
        );

        // Add the view as a symbol
        parsed.add_symbol(Symbol {
            name: view_name.clone(),
            qualified_name: format!("view:{}", view_name),
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

    fn extract_view_name(&self, path: &str) -> String {
        path.replace("resources/views/", "")
            .replace(".blade.php", "")
            .replace('/', ".")
    }

    fn extract_extends(&self, content: &str) -> Option<String> {
        self.extends_regex.captures(content)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn extract_sections(&self, content: &str) -> Vec<String> {
        self.section_regex.captures_iter(content)
            .filter_map(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn extract_yields(&self, content: &str) -> Vec<String> {
        self.yield_regex.captures_iter(content)
            .filter_map(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn extract_includes(&self, content: &str) -> Vec<String> {
        let mut includes = Vec::new();

        // Regular includes
        for caps in self.include_regex.captures_iter(content) {
            if let Some(view) = caps.get(1) {
                let name = view.as_str().to_string();
                if !includes.contains(&name) {
                    includes.push(name);
                }
            }
        }

        // Include if
        for caps in self.include_if_regex.captures_iter(content) {
            if let Some(view) = caps.get(1) {
                let name = view.as_str().to_string();
                if !includes.contains(&name) {
                    includes.push(name);
                }
            }
        }

        // Include when
        for caps in self.include_when_regex.captures_iter(content) {
            if let Some(view) = caps.get(1) {
                let name = view.as_str().to_string();
                if !includes.contains(&name) {
                    includes.push(name);
                }
            }
        }

        // Include first
        for caps in self.include_first_regex.captures_iter(content) {
            if let Some(list) = caps.get(1) {
                for item in list.as_str().split(',') {
                    let name = item.trim().trim_matches(|c| c == '\'' || c == '"').to_string();
                    if !name.is_empty() && !includes.contains(&name) {
                        includes.push(name);
                    }
                }
            }
        }

        // Each
        for caps in self.each_regex.captures_iter(content) {
            if let Some(view) = caps.get(1) {
                let name = view.as_str().to_string();
                if !includes.contains(&name) {
                    includes.push(name);
                }
            }
        }

        includes
    }

    fn extract_components(&self, content: &str) -> Vec<serde_json::Value> {
        let mut components = Vec::new();

        // Class-based components
        for caps in self.component_class_regex.captures_iter(content) {
            if let Some(name) = caps.get(1) {
                components.push(serde_json::json!({
                    "name": name.as_str(),
                    "type": "class"
                }));
            }
        }

        // X-components
        for caps in self.component_x_regex.captures_iter(content) {
            if let Some(name) = caps.get(1) {
                let normalized = name.as_str().replace('-', ".");
                components.push(serde_json::json!({
                    "name": normalized,
                    "type": "anonymous"
                }));
            }
        }

        // Dynamic components
        for caps in self.component_anonymous_regex.captures_iter(content) {
            if let Some(name) = caps.get(1) {
                components.push(serde_json::json!({
                    "name": name.as_str(),
                    "type": "dynamic"
                }));
            }
        }

        components
    }

    fn extract_slots(&self, content: &str) -> Vec<String> {
        let mut slots = Vec::new();

        for caps in self.slot_regex.captures_iter(content) {
            let slot_name = caps.get(1).or_else(|| caps.get(2));
            if let Some(name) = slot_name {
                let n = name.as_str().to_string();
                if !slots.contains(&n) {
                    slots.push(n);
                }
            }
        }

        slots
    }

    fn extract_stacks(&self, content: &str) -> Vec<String> {
        self.stack_regex.captures_iter(content)
            .filter_map(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn extract_pushes(&self, content: &str) -> Vec<String> {
        let mut pushes = Vec::new();

        for caps in self.push_regex.captures_iter(content) {
            if let Some(name) = caps.get(1) {
                let n = name.as_str().to_string();
                if !pushes.contains(&n) {
                    pushes.push(n);
                }
            }
        }

        for caps in self.prepend_regex.captures_iter(content) {
            if let Some(name) = caps.get(1) {
                let n = name.as_str().to_string();
                if !pushes.contains(&n) {
                    pushes.push(n);
                }
            }
        }

        pushes
    }

    fn extract_props(&self, content: &str) -> Vec<String> {
        let mut props = Vec::new();

        for caps in self.props_regex.captures_iter(content) {
            if let Some(prop_list) = caps.get(1) {
                let prop_regex = Regex::new(r#"['"](\w+)['"]"#).unwrap();
                for prop_caps in prop_regex.captures_iter(prop_list.as_str()) {
                    if let Some(prop) = prop_caps.get(1) {
                        let name = prop.as_str().to_string();
                        if !props.contains(&name) {
                            props.push(name);
                        }
                    }
                }
            }
        }

        props
    }

    fn extract_livewire_components(&self, content: &str) -> Vec<String> {
        let mut components = Vec::new();

        for caps in self.livewire_regex.captures_iter(content) {
            let name = caps.get(1).or_else(|| caps.get(2));
            if let Some(n) = name {
                let component = n.as_str().to_string();
                if !components.contains(&component) {
                    components.push(component);
                }
            }
        }

        components
    }

    fn extract_permissions(&self, content: &str) -> Vec<String> {
        let mut permissions = Vec::new();

        for caps in self.can_regex.captures_iter(content) {
            if let Some(perm) = caps.get(1) {
                let name = perm.as_str().to_string();
                if !permissions.contains(&name) {
                    permissions.push(name);
                }
            }
        }

        for caps in self.cannot_regex.captures_iter(content) {
            if let Some(perm) = caps.get(1) {
                let name = perm.as_str().to_string();
                if !permissions.contains(&name) {
                    permissions.push(name);
                }
            }
        }

        permissions
    }

    fn extract_error_bags(&self, content: &str) -> Vec<String> {
        self.error_regex.captures_iter(content)
            .filter_map(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn count_directives(&self, content: &str) -> serde_json::Value {
        let mut counts = serde_json::Map::new();

        let directives: [(&str, &Regex); 10] = [
            ("if", &self.if_regex),
            ("foreach", &self.foreach_regex),
            ("for", &self.for_regex),
            ("while", &self.while_regex),
            ("forelse", &self.forelse_regex),
            ("switch", &self.switch_regex),
            ("auth", &self.auth_regex),
            ("guest", &self.guest_regex),
            ("php", &self.php_regex),
            ("csrf", &self.csrf_regex),
        ];

        for (name, regex) in directives {
            let count = regex.find_iter(content).count();
            if count > 0 {
                counts.insert(name.to_string(), serde_json::json!(count));
            }
        }

        // Count echo expressions
        let echo_count = self.echo_regex.find_iter(content).count();
        if echo_count > 0 {
            counts.insert("echo".to_string(), serde_json::json!(echo_count));
        }

        // Count raw echo
        let raw_count = self.raw_echo_regex.find_iter(content).count();
        if raw_count > 0 {
            counts.insert("raw_echo".to_string(), serde_json::json!(raw_count));
        }

        serde_json::Value::Object(counts)
    }
}

impl Default for BladeParser {
    fn default() -> Self {
        Self::new()
    }
}
