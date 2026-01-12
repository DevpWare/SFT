use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

use crate::core::{ParserInfo, ProjectType};
use crate::models::{
    ParseResult, ParsedFile, SourceFile, UnifiedEdge, UnifiedEdgeType, UnifiedNode,
    UnifiedNodeType,
};
use crate::parsers::common::{generate_id, scan_directory};
use crate::parsers::{
    ParserCapabilities, ParserConfig, ParserResult, ProgressCallback, ProjectParser,
};

use super::blade_parser::BladeParser;
use super::controller_parser::ControllerParser;
use super::inertia_parser::InertiaParser;
use super::migration_parser::MigrationParser;
use super::model_parser::ModelParser;
use super::php_parser::PhpParser;
use super::route_parser::RouteParser;

/// Laravel PHP framework parser
pub struct LaravelParser {
    php_parser: PhpParser,
    controller_parser: ControllerParser,
    model_parser: ModelParser,
    route_parser: RouteParser,
    migration_parser: MigrationParser,
    blade_parser: BladeParser,
    inertia_parser: InertiaParser,
}

impl LaravelParser {
    pub fn new() -> Self {
        Self {
            php_parser: PhpParser::new(),
            controller_parser: ControllerParser::new(),
            model_parser: ModelParser::new(),
            route_parser: RouteParser::new(),
            migration_parser: MigrationParser::new(),
            blade_parser: BladeParser::new(),
            inertia_parser: InertiaParser::new(),
        }
    }

    /// Determine the file type based on path and content hints
    fn determine_file_type(&self, file: &SourceFile) -> LaravelFileType {
        let path = &file.path;
        let name = &file.name;

        // Normalize path separators for cross-platform compatibility
        let normalized_path = path.replace('\\', "/");
        let path_lower = normalized_path.to_lowercase();

        // Inertia pages (Vue, React, Svelte in resources/js/Pages)
        if (path_lower.contains("/resources/js/pages/") || path_lower.contains("/resources/js/Pages/"))
            && (name.ends_with(".vue") || name.ends_with(".jsx") || name.ends_with(".tsx") || name.ends_with(".svelte"))
        {
            return LaravelFileType::InertiaPage;
        }

        // Blade templates (check first, most specific)
        if name.ends_with(".blade.php") {
            return LaravelFileType::BladeView;
        }

        // Route files
        if path_lower.contains("/routes/") && name.ends_with(".php") {
            return LaravelFileType::Route;
        }

        // Migrations (check before other patterns, very specific path)
        if path_lower.contains("/migrations/") || path_lower.contains("/database/migrations") {
            return LaravelFileType::Migration;
        }

        // Controllers (check by path or name pattern)
        if path_lower.contains("/controllers/") || name.ends_with("Controller.php") {
            return LaravelFileType::Controller;
        }

        // Middleware
        if path_lower.contains("/middleware/") {
            return LaravelFileType::Middleware;
        }

        // Requests (Form Requests)
        if path_lower.contains("/requests/") {
            return LaravelFileType::Request;
        }

        // Resources (API Resources) - but not views/resources
        if path_lower.contains("/resources/") && !path_lower.contains("/views/") && !path_lower.contains("resources/views") {
            return LaravelFileType::Resource;
        }

        // Providers
        if path_lower.contains("/providers/") {
            return LaravelFileType::Provider;
        }

        // Events
        if path_lower.contains("/events/") {
            return LaravelFileType::Event;
        }

        // Listeners
        if path_lower.contains("/listeners/") {
            return LaravelFileType::Listener;
        }

        // Jobs
        if path_lower.contains("/jobs/") {
            return LaravelFileType::Job;
        }

        // Policies
        if path_lower.contains("/policies/") {
            return LaravelFileType::Policy;
        }

        // Commands
        if path_lower.contains("/commands/") {
            return LaravelFileType::Command;
        }

        // Console (Artisan commands)
        if path_lower.contains("/console/") && !path_lower.contains("/console.php") {
            return LaravelFileType::Command;
        }

        // Config files
        if path_lower.contains("/config/") {
            return LaravelFileType::Config;
        }

        // Seeders
        if path_lower.contains("/seeders/") || path_lower.contains("/database/seeders") {
            return LaravelFileType::Seeder;
        }

        // Factories
        if path_lower.contains("/factories/") || path_lower.contains("/database/factories") {
            return LaravelFileType::Factory;
        }

        // Tests
        if path_lower.contains("/tests/") {
            return LaravelFileType::Test;
        }

        // Services
        if path_lower.contains("/services/") {
            return LaravelFileType::Service;
        }

        // Repositories
        if path_lower.contains("/repositories/") {
            return LaravelFileType::Repository;
        }

        // Actions
        if path_lower.contains("/actions/") {
            return LaravelFileType::Action;
        }

        // DTOs
        if path_lower.contains("/dto/") || path_lower.contains("/dtos/") {
            return LaravelFileType::Dto;
        }

        // Notifications
        if path_lower.contains("/notifications/") {
            return LaravelFileType::Notification;
        }

        // Mail/Mailables
        if path_lower.contains("/mail/") {
            return LaravelFileType::Mailable;
        }

        // Observers
        if path_lower.contains("/observers/") {
            return LaravelFileType::Observer;
        }

        // Scopes
        if path_lower.contains("/scopes/") {
            return LaravelFileType::Scope;
        }

        // Rules
        if path_lower.contains("/rules/") {
            return LaravelFileType::Rule;
        }

        // Casts
        if path_lower.contains("/casts/") {
            return LaravelFileType::Cast;
        }

        // Contracts/Interfaces
        if path_lower.contains("/contracts/") || path_lower.contains("/interfaces/") {
            return LaravelFileType::Contract;
        }

        // Exceptions
        if path_lower.contains("/exceptions/") {
            return LaravelFileType::Exception;
        }

        // Enums
        if path_lower.contains("/enums/") {
            return LaravelFileType::Enum;
        }

        // Traits
        if path_lower.contains("/traits/") || path_lower.contains("/concerns/") {
            return LaravelFileType::Trait;
        }

        // Models - check explicitly for /Models/ directory first
        if path_lower.contains("/models/") {
            return LaravelFileType::Model;
        }

        // Default to generic PHP - will be refined later based on extends/implements
        LaravelFileType::Php
    }

    /// Refine file type based on class extends/implements from parsed symbols
    fn refine_file_type(&self, initial_type: LaravelFileType, parsed_file: &ParsedFile) -> LaravelFileType {
        // If already a specific type (not generic Php), keep it
        if initial_type != LaravelFileType::Php {
            return initial_type;
        }

        // Look at the main class symbol to determine type
        for symbol in &parsed_file.symbols {
            if symbol.symbol_type != crate::models::SymbolType::Class {
                continue;
            }

            // Check extends
            if let Some(ref extends) = symbol.extends {
                let parent = extends.rsplit('\\').next().unwrap_or(extends);

                match parent {
                    // Controllers
                    "Controller" | "BaseController" | "ResourceController" => {
                        return LaravelFileType::Controller;
                    }
                    // Models
                    "Model" | "Authenticatable" | "Pivot" => {
                        return LaravelFileType::Model;
                    }
                    // Form Requests
                    "FormRequest" | "Request" => {
                        return LaravelFileType::Request;
                    }
                    // API Resources
                    "JsonResource" | "Resource" | "ResourceCollection" => {
                        return LaravelFileType::Resource;
                    }
                    // Service Providers
                    "ServiceProvider" | "AppServiceProvider" | "RouteServiceProvider" => {
                        return LaravelFileType::Provider;
                    }
                    // Events
                    "Event" => {
                        return LaravelFileType::Event;
                    }
                    // Jobs
                    "Job" => {
                        return LaravelFileType::Job;
                    }
                    // Commands
                    "Command" | "ConsoleCommand" => {
                        return LaravelFileType::Command;
                    }
                    // Notifications
                    "Notification" => {
                        return LaravelFileType::Notification;
                    }
                    // Mailables
                    "Mailable" => {
                        return LaravelFileType::Mailable;
                    }
                    // Migrations
                    "Migration" => {
                        return LaravelFileType::Migration;
                    }
                    // Seeders
                    "Seeder" => {
                        return LaravelFileType::Seeder;
                    }
                    // Factories
                    "Factory" => {
                        return LaravelFileType::Factory;
                    }
                    // Exceptions
                    "Exception" | "HttpException" | "ValidationException" => {
                        return LaravelFileType::Exception;
                    }
                    // Policies
                    "Policy" => {
                        return LaravelFileType::Policy;
                    }
                    // Middleware
                    "Middleware" => {
                        return LaravelFileType::Middleware;
                    }
                    // Casts
                    "CastsAttributes" => {
                        return LaravelFileType::Cast;
                    }
                    _ => {}
                }
            }

            // Check implements
            if let Some(ref implements) = symbol.implements {
                for iface in implements {
                    let iface_name = iface.rsplit('\\').next().unwrap_or(iface);

                    match iface_name {
                        // Jobs
                        "ShouldQueue" => {
                            // Could be Job, Notification, or Mailable with queue
                            // If extends wasn't matched, assume Job
                            if symbol.extends.is_none() {
                                return LaravelFileType::Job;
                            }
                        }
                        // Listeners
                        "ShouldHandleEventsAfterCommit" | "Listener" => {
                            return LaravelFileType::Listener;
                        }
                        // Rules
                        "Rule" | "ValidationRule" | "DataAwareRule" | "ValidatorAwareRule" => {
                            return LaravelFileType::Rule;
                        }
                        // Casts
                        "CastsAttributes" | "CastsInboundAttributes" => {
                            return LaravelFileType::Cast;
                        }
                        // Middleware
                        "MiddlewareInterface" | "Middleware" => {
                            return LaravelFileType::Middleware;
                        }
                        _ => {}
                    }
                }
            }

            // Check by class name suffix/pattern
            let class_name = &symbol.name;
            if class_name.ends_with("Service") {
                return LaravelFileType::Service;
            }
            if class_name.ends_with("Repository") {
                return LaravelFileType::Repository;
            }
            if class_name.ends_with("Action") {
                return LaravelFileType::Action;
            }
            if class_name.ends_with("DTO") || class_name.ends_with("Dto") {
                return LaravelFileType::Dto;
            }
            if class_name.ends_with("Observer") {
                return LaravelFileType::Observer;
            }
            if class_name.ends_with("Scope") {
                return LaravelFileType::Scope;
            }
            if class_name.ends_with("Policy") {
                return LaravelFileType::Policy;
            }
            if class_name.ends_with("Listener") {
                return LaravelFileType::Listener;
            }
            if class_name.ends_with("Event") {
                return LaravelFileType::Event;
            }
            if class_name.ends_with("Job") {
                return LaravelFileType::Job;
            }
            if class_name.ends_with("Controller") {
                return LaravelFileType::Controller;
            }
            if class_name.ends_with("Request") {
                return LaravelFileType::Request;
            }
            if class_name.ends_with("Resource") {
                return LaravelFileType::Resource;
            }
            if class_name.ends_with("Notification") {
                return LaravelFileType::Notification;
            }
            if class_name.ends_with("Mail") || class_name.ends_with("Mailable") {
                return LaravelFileType::Mailable;
            }
            if class_name.ends_with("Exception") {
                return LaravelFileType::Exception;
            }
            if class_name.ends_with("Middleware") {
                return LaravelFileType::Middleware;
            }
            if class_name.ends_with("Cast") {
                return LaravelFileType::Cast;
            }
            if class_name.ends_with("Rule") {
                return LaravelFileType::Rule;
            }
            if class_name.ends_with("Provider") {
                return LaravelFileType::Provider;
            }
            if class_name.ends_with("Seeder") {
                return LaravelFileType::Seeder;
            }
            if class_name.ends_with("Factory") {
                return LaravelFileType::Factory;
            }
            if class_name.ends_with("Command") {
                return LaravelFileType::Command;
            }
        }

        // Check for interfaces
        for symbol in &parsed_file.symbols {
            if symbol.symbol_type == crate::models::SymbolType::Interface {
                return LaravelFileType::Interface;
            }
        }

        // Check for traits
        for symbol in &parsed_file.symbols {
            if symbol.symbol_type == crate::models::SymbolType::Trait {
                return LaravelFileType::Trait;
            }
        }

        // Keep original type
        initial_type
    }
}

/// Types of Laravel files
#[derive(Debug, Clone, PartialEq)]
enum LaravelFileType {
    Controller,
    Model,
    BladeView,
    Route,
    Migration,
    Middleware,
    Request,
    Resource,
    Provider,
    Event,
    Listener,
    Job,
    Policy,
    Command,
    Config,
    Seeder,
    Factory,
    Test,
    InertiaPage,
    // Additional types based on extends/implements/namespace
    Service,
    Repository,
    Action,
    Dto,
    Notification,
    Mailable,
    Observer,
    Scope,
    Rule,
    Cast,
    Contract,
    Exception,
    Enum,
    Trait,
    Interface,
    Php,
}

impl Default for LaravelParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProjectParser for LaravelParser {
    fn info(&self) -> ParserInfo {
        ParserInfo {
            id: "laravel".to_string(),
            display_name: "Laravel (PHP)".to_string(),
            description: "Parser for Laravel PHP framework projects".to_string(),
            version: "0.2.0".to_string(),
            file_extensions: vec!["php".to_string(), "blade.php".to_string()],
            marker_files: vec!["composer.json".to_string(), "artisan".to_string()],
            marker_dirs: vec![
                "app/Http/Controllers".to_string(),
                "resources/views".to_string(),
            ],
            project_type: ProjectType::Laravel,
            primary_color: "#FF2D20".to_string(),
            is_available: true,
        }
    }

    fn default_config(&self) -> ParserConfig {
        ParserConfig {
            include_extensions: vec!["php".to_string()],
            exclude_dirs: vec![
                "vendor".to_string(),
                "node_modules".to_string(),
                "storage".to_string(),
                "bootstrap/cache".to_string(),
                ".git".to_string(),
            ],
            encoding: "utf-8".to_string(),
            parse_external_deps: false,
            max_depth: None,
            language_options: Default::default(),
        }
    }

    fn capabilities(&self) -> ParserCapabilities {
        ParserCapabilities {
            node_types: vec![
                "controller".to_string(),
                "model".to_string(),
                "view".to_string(),
                "route".to_string(),
                "migration".to_string(),
                "middleware".to_string(),
                "provider".to_string(),
                "event".to_string(),
                "listener".to_string(),
                "job".to_string(),
                "policy".to_string(),
                "command".to_string(),
            ],
            edge_types: vec![
                "uses".to_string(),
                "extends".to_string(),
                "implements".to_string(),
                "routes_to".to_string(),
                "renders".to_string(),
                "has_many".to_string(),
                "belongs_to".to_string(),
                "middleware".to_string(),
            ],
            supports_incremental: false,
            supports_cancellation: true,
            available_metrics: vec![
                "lines_of_code".to_string(),
                "routes_count".to_string(),
                "models_count".to_string(),
                "controllers_count".to_string(),
            ],
        }
    }

    fn detect_confidence(&self, root_path: &Path) -> f32 {
        let mut score = 0.0f32;

        // Check for composer.json with laravel/framework
        if let Ok(content) = std::fs::read_to_string(root_path.join("composer.json")) {
            if content.contains("laravel/framework") {
                score += 0.6;
            }
        }

        // Check for artisan
        if root_path.join("artisan").exists() {
            score += 0.2;
        }

        // Check for Laravel directories
        let laravel_dirs = [
            "app/Http/Controllers",
            "resources/views",
            "routes",
            "database/migrations",
        ];
        for dir in laravel_dirs {
            if root_path.join(dir).is_dir() {
                score += 0.05;
            }
        }

        score.min(1.0)
    }

    fn can_handle_file(&self, file_path: &Path) -> bool {
        file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("php"))
            .unwrap_or(false)
    }

    async fn scan_files(
        &self,
        root_path: &Path,
        config: &ParserConfig,
        _progress: Option<ProgressCallback>,
    ) -> ParserResult<Vec<SourceFile>> {
        // Include PHP and Inertia files (Vue, React, Svelte)
        let extensions: Vec<&str> = vec!["php", "vue", "jsx", "tsx", "svelte"];
        let exclude_dirs: Vec<&str> = config.exclude_dirs.iter().map(|s| s.as_str()).collect();

        Ok(scan_directory(root_path, &extensions, &exclude_dirs))
    }

    async fn parse_file(
        &self,
        file: &SourceFile,
        config: &ParserConfig,
    ) -> ParserResult<ParsedFile> {
        let file_type = self.determine_file_type(file);

        // Use specialized parser based on file type
        match file_type {
            LaravelFileType::Controller => self.controller_parser.parse(file, config).await,
            LaravelFileType::Model => self.model_parser.parse(file, config).await,
            LaravelFileType::BladeView => self.blade_parser.parse(file, config).await,
            LaravelFileType::Route => self.route_parser.parse(file, config).await,
            LaravelFileType::Migration => self.migration_parser.parse(file, config).await,
            LaravelFileType::InertiaPage => self.inertia_parser.parse(file, config).await,
            // For other file types, use the base PHP parser with type annotation
            _ => {
                let mut parsed = self.php_parser.parse(file, config).await?;
                parsed.metadata.insert(
                    "laravel_type".to_string(),
                    serde_json::Value::String(format!("{:?}", file_type)),
                );
                Ok(parsed)
            }
        }
    }

    fn generate_nodes(&self, parse_result: &ParseResult) -> Vec<UnifiedNode> {
        let mut nodes = Vec::new();

        for parsed_file in &parse_result.files {
            // Get initial type from path, then refine using extends/implements
            let initial_type = self.determine_file_type(&parsed_file.source);
            let file_type = self.refine_file_type(initial_type, parsed_file);

            // Determine node type based on Laravel file type
            let node_type = match file_type {
                LaravelFileType::Controller => UnifiedNodeType::Controller,
                LaravelFileType::Model => UnifiedNodeType::Model,
                LaravelFileType::BladeView => UnifiedNodeType::View,
                LaravelFileType::Route => UnifiedNodeType::Route,
                LaravelFileType::Migration => UnifiedNodeType::Migration,
                LaravelFileType::Middleware => UnifiedNodeType::Middleware,
                LaravelFileType::Request => UnifiedNodeType::Custom("request".to_string()),
                LaravelFileType::Resource => UnifiedNodeType::Custom("resource".to_string()),
                LaravelFileType::Provider => UnifiedNodeType::Custom("provider".to_string()),
                LaravelFileType::Event => UnifiedNodeType::Custom("event".to_string()),
                LaravelFileType::Listener => UnifiedNodeType::Custom("listener".to_string()),
                LaravelFileType::Job => UnifiedNodeType::Custom("job".to_string()),
                LaravelFileType::Policy => UnifiedNodeType::Custom("policy".to_string()),
                LaravelFileType::Command => UnifiedNodeType::Custom("command".to_string()),
                LaravelFileType::Config => UnifiedNodeType::ConfigFile,
                LaravelFileType::Seeder => UnifiedNodeType::Custom("seeder".to_string()),
                LaravelFileType::Factory => UnifiedNodeType::Custom("factory".to_string()),
                LaravelFileType::Test => UnifiedNodeType::Custom("test".to_string()),
                LaravelFileType::InertiaPage => UnifiedNodeType::Component,
                // New types
                LaravelFileType::Service => UnifiedNodeType::Custom("service".to_string()),
                LaravelFileType::Repository => UnifiedNodeType::Custom("repository".to_string()),
                LaravelFileType::Action => UnifiedNodeType::Custom("action".to_string()),
                LaravelFileType::Dto => UnifiedNodeType::Custom("dto".to_string()),
                LaravelFileType::Notification => UnifiedNodeType::Custom("notification".to_string()),
                LaravelFileType::Mailable => UnifiedNodeType::Custom("mailable".to_string()),
                LaravelFileType::Observer => UnifiedNodeType::Custom("observer".to_string()),
                LaravelFileType::Scope => UnifiedNodeType::Custom("scope".to_string()),
                LaravelFileType::Rule => UnifiedNodeType::Custom("rule".to_string()),
                LaravelFileType::Cast => UnifiedNodeType::Custom("cast".to_string()),
                LaravelFileType::Contract => UnifiedNodeType::Interface,
                LaravelFileType::Exception => UnifiedNodeType::Custom("exception".to_string()),
                LaravelFileType::Enum => UnifiedNodeType::Custom("enum".to_string()),
                LaravelFileType::Trait => UnifiedNodeType::Trait,
                LaravelFileType::Interface => UnifiedNodeType::Interface,
                LaravelFileType::Php => UnifiedNodeType::SourceFile,
            };

            // Create file node
            let file_id = generate_id(&parsed_file.source.path);
            let mut file_node =
                UnifiedNode::new(file_id.clone(), node_type.clone(), parsed_file.source.name.clone())
                    .with_file(parsed_file.source.path.clone())
                    .with_language("php");

            // Set qualified name from metadata if available
            if let Some(namespace) = parsed_file.metadata.get("namespace") {
                if let Some(ns) = namespace.as_str() {
                    file_node.qualified_name = ns.to_string();
                }
            } else if let Some(view_name) = parsed_file.metadata.get("view_name") {
                // For Blade views, use view:{name} format for edge matching
                if let Some(name) = view_name.as_str() {
                    file_node.qualified_name = format!("view:{}", name);
                }
            } else if let Some(page_name) = parsed_file.metadata.get("page_name") {
                // For Inertia pages, use inertia:{name} format for edge matching
                if let Some(name) = page_name.as_str() {
                    file_node.qualified_name = format!("inertia:{}", name);
                }
            }

            // Set size based on file type importance
            file_node.size = match file_type {
                LaravelFileType::Controller => 8,
                LaravelFileType::Model => 7,
                LaravelFileType::Service => 7,
                LaravelFileType::Repository => 7,
                LaravelFileType::Route => 6,
                LaravelFileType::BladeView => 5,
                LaravelFileType::InertiaPage => 6,
                LaravelFileType::Migration => 5,
                LaravelFileType::Middleware => 6,
                LaravelFileType::Provider => 6,
                LaravelFileType::Action => 6,
                LaravelFileType::Request => 5,
                LaravelFileType::Resource => 5,
                LaravelFileType::Event => 5,
                LaravelFileType::Listener => 5,
                LaravelFileType::Job => 5,
                LaravelFileType::Policy => 5,
                LaravelFileType::Command => 5,
                LaravelFileType::Notification => 5,
                LaravelFileType::Mailable => 5,
                LaravelFileType::Observer => 5,
                LaravelFileType::Rule => 4,
                LaravelFileType::Scope => 4,
                LaravelFileType::Cast => 4,
                LaravelFileType::Dto => 4,
                LaravelFileType::Contract => 5,
                LaravelFileType::Interface => 5,
                LaravelFileType::Trait => 5,
                LaravelFileType::Exception => 4,
                LaravelFileType::Enum => 4,
                LaravelFileType::Config => 4,
                LaravelFileType::Seeder => 4,
                LaravelFileType::Factory => 4,
                LaravelFileType::Test => 4,
                LaravelFileType::Php => 3,
            };

            // Add extra metadata
            file_node.metadata.extra = parsed_file
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            nodes.push(file_node);

            // Create nodes for classes/symbols within the file
            for symbol in &parsed_file.symbols {
                let symbol_node_type = match symbol.symbol_type {
                    crate::models::SymbolType::Class => {
                        // Refine based on Laravel context
                        if file_type == LaravelFileType::Controller {
                            UnifiedNodeType::Controller
                        } else if file_type == LaravelFileType::Model {
                            UnifiedNodeType::Model
                        } else {
                            UnifiedNodeType::Class
                        }
                    }
                    crate::models::SymbolType::Interface => UnifiedNodeType::Interface,
                    crate::models::SymbolType::Trait => UnifiedNodeType::Trait,
                    crate::models::SymbolType::Method => UnifiedNodeType::Method,
                    crate::models::SymbolType::Function => UnifiedNodeType::Function,
                    _ => continue, // Skip other symbols for now
                };

                let symbol_id = generate_id(&format!("{}::{}", parsed_file.source.path, symbol.name));

                let mut symbol_node = UnifiedNode::new(
                    symbol_id,
                    symbol_node_type,
                    symbol.name.clone(),
                )
                .with_file(parsed_file.source.path.clone())
                .with_language("php");

                symbol_node.qualified_name = symbol.qualified_name.clone();

                if let Some(ref vis) = symbol.visibility {
                    symbol_node.metadata.visibility = Some(vis.clone());
                }

                symbol_node.metadata.is_abstract = symbol.is_abstract;
                symbol_node.metadata.is_static = symbol.is_static;

                if let Some(ref parent) = symbol.extends {
                    symbol_node.metadata.parent_class = Some(parent.clone());
                }

                symbol_node.metadata.implements = symbol.implements.clone();

                nodes.push(symbol_node);
            }
        }

        nodes
    }

    fn generate_edges(
        &self,
        parse_result: &ParseResult,
        nodes: &[UnifiedNode],
    ) -> Vec<UnifiedEdge> {
        let mut edges = Vec::new();

        // Build lookup maps for faster edge creation
        let node_by_name: HashMap<&str, &UnifiedNode> = nodes
            .iter()
            .map(|n| (n.name.as_str(), n))
            .collect();

        let node_by_qualified: HashMap<&str, &UnifiedNode> = nodes
            .iter()
            .map(|n| (n.qualified_name.as_str(), n))
            .collect();

        for parsed_file in &parse_result.files {
            let source_id = generate_id(&parsed_file.source.path);

            // Create edges from dependencies (use statements)
            for dep in &parsed_file.dependencies {
                // Try to find target node by qualified name
                let target_name = dep.target.rsplit('\\').next().unwrap_or(&dep.target);

                if let Some(target_node) = node_by_name.get(target_name) {
                    edges.push(UnifiedEdge::new(
                        source_id.clone(),
                        target_node.id.clone(),
                        UnifiedEdgeType::Uses,
                    ));
                }
            }

            // Create edges from relationships (for models)
            if let Some(relationships) = parsed_file.metadata.get("relationships") {
                if let Some(rels) = relationships.as_array() {
                    for rel in rels {
                        if let (Some(rel_type), Some(related_model)) =
                            (rel.get("type"), rel.get("related_model"))
                        {
                            let rel_type_str = rel_type.as_str().unwrap_or("");
                            let model_name = related_model.as_str().unwrap_or("");

                            if let Some(target_node) = node_by_name.get(model_name) {
                                let edge_type = match rel_type_str {
                                    "hasMany" | "hasManyThrough" => {
                                        UnifiedEdgeType::Custom("has_many".to_string())
                                    }
                                    "hasOne" | "hasOneThrough" => {
                                        UnifiedEdgeType::Custom("has_one".to_string())
                                    }
                                    "belongsTo" => {
                                        UnifiedEdgeType::Custom("belongs_to".to_string())
                                    }
                                    "belongsToMany" => {
                                        UnifiedEdgeType::Custom("belongs_to_many".to_string())
                                    }
                                    "morphTo" | "morphOne" | "morphMany" => {
                                        UnifiedEdgeType::Custom("morph".to_string())
                                    }
                                    _ => UnifiedEdgeType::Uses,
                                };

                                edges.push(UnifiedEdge::new(
                                    source_id.clone(),
                                    target_node.id.clone(),
                                    edge_type,
                                ));
                            }
                        }
                    }
                }
            }

            // Create edges from controller to views
            if let Some(views) = parsed_file.metadata.get("views_referenced") {
                if let Some(view_list) = views.as_array() {
                    for view in view_list {
                        if let Some(view_name) = view.as_str() {
                            // Try to find the view node
                            let view_path = format!("view:{}", view_name);
                            if let Some(target_node) = node_by_qualified.get(view_path.as_str()) {
                                edges.push(UnifiedEdge::new(
                                    source_id.clone(),
                                    target_node.id.clone(),
                                    UnifiedEdgeType::Custom("renders".to_string()),
                                ));
                            }
                        }
                    }
                }
            }

            // Create edges from controller to Inertia pages
            if let Some(pages) = parsed_file.metadata.get("inertia_pages") {
                if let Some(page_list) = pages.as_array() {
                    for page in page_list {
                        if let Some(page_name) = page.as_str() {
                            // Try to find the Inertia page node
                            let page_path = format!("inertia:{}", page_name);
                            if let Some(target_node) = node_by_qualified.get(page_path.as_str()) {
                                edges.push(UnifiedEdge::new(
                                    source_id.clone(),
                                    target_node.id.clone(),
                                    UnifiedEdgeType::Custom("renders".to_string()),
                                ));
                            }
                        }
                    }
                }
            }

            // Create edges from routes to controllers
            if let Some(routes) = parsed_file.metadata.get("routes") {
                if let Some(route_list) = routes.as_array() {
                    for route in route_list {
                        if let Some(action) = route.get("action") {
                            if let Some(controller) = action.get("controller") {
                                if let Some(controller_name) = controller.as_str() {
                                    if let Some(target_node) = node_by_name.get(controller_name) {
                                        edges.push(UnifiedEdge::new(
                                            source_id.clone(),
                                            target_node.id.clone(),
                                            UnifiedEdgeType::Custom("routes_to".to_string()),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Create edges from Blade extends
            if let Some(extends) = parsed_file.metadata.get("extends") {
                if let Some(parent_view) = extends.as_str() {
                    let parent_path = format!("view:{}", parent_view);
                    if let Some(target_node) = node_by_qualified.get(parent_path.as_str()) {
                        edges.push(UnifiedEdge::new(
                            source_id.clone(),
                            target_node.id.clone(),
                            UnifiedEdgeType::Extends,
                        ));
                    }
                }
            }

            // Create edges from Blade includes
            if let Some(includes) = parsed_file.metadata.get("includes") {
                if let Some(include_list) = includes.as_array() {
                    for include in include_list {
                        if let Some(include_name) = include.as_str() {
                            let include_path = format!("view:{}", include_name);
                            if let Some(target_node) = node_by_qualified.get(include_path.as_str()) {
                                edges.push(UnifiedEdge::new(
                                    source_id.clone(),
                                    target_node.id.clone(),
                                    UnifiedEdgeType::Custom("includes".to_string()),
                                ));
                            }
                        }
                    }
                }
            }

            // Create edges from foreign keys (migrations)
            if let Some(foreign_keys) = parsed_file.metadata.get("foreign_keys") {
                if let Some(fk_list) = foreign_keys.as_array() {
                    for fk in fk_list {
                        if let Some(on_table) = fk.get("on_table") {
                            if let Some(table_name) = on_table.as_str() {
                                // Try to find a migration that creates this table
                                for node in nodes {
                                    if node.node_type == UnifiedNodeType::Migration {
                                        if let Some(tables) = node.metadata.extra.get("tables_created") {
                                            if let Some(tables_arr) = tables.as_array() {
                                                if tables_arr.iter().any(|t| t.as_str() == Some(table_name)) {
                                                    edges.push(UnifiedEdge::new(
                                                        source_id.clone(),
                                                        node.id.clone(),
                                                        UnifiedEdgeType::Custom("references".to_string()),
                                                    ));
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        edges
    }

    fn detect_file_pairs(&self, _files: &[SourceFile]) -> Vec<(String, String)> {
        // Laravel doesn't have strict file pairs like Delphi
        // But we could detect controller-view relationships here
        Vec::new()
    }
}
