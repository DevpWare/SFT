// Unified types for graph nodes and edges - language independent

export type UnifiedNodeType =
  | "source_file"
  | "config_file"
  | "form_file"
  | "module"
  | "class"
  | "interface"
  | "trait"
  | "enum"
  | "struct"
  | "function"
  | "method"
  | "constructor"
  | "destructor"
  | "component"
  | "form"
  | "page"
  | "view"
  | "route"
  | "controller"
  | "middleware"
  | "model"
  | "migration"
  | "table"
  | "query"
  | "package"
  | "variable"
  | "constant"
  | { custom: string };

export type UnifiedEdgeType =
  | "uses"
  | "extends"
  | "implements"
  | "includes"
  | "contains"
  | "defines"
  | "belongs_to"
  | "calls"
  | "instantiates"
  | "file_pair"
  | "routes"
  | "renders"
  | "queries_table"
  | "has_relation"
  | "references"
  | { custom: string };

export type ProjectType =
  | "delphi"
  | "laravel"
  | "nodejs"
  | "php"
  | "csharp"
  | "java"
  | "python"
  | "go"
  | "rust_lang"
  | "unknown";

export interface Position3D {
  x: number;
  y: number;
  z: number;
}

export interface NodeMetadata {
  size_bytes?: number;
  visibility?: string;
  is_abstract?: boolean;
  is_static?: boolean;
  return_type?: string;
  parent_class?: string;
  implements?: string[];
  uses_traits?: string[];
  documentation?: string;
  notes?: string;
  tags?: string[];
  status?: "ok" | "review" | "deprecated" | "critical" | "todo";
  [key: string]: unknown;
}

export interface UnifiedNode {
  id: string;
  node_type: UnifiedNodeType;
  name: string;
  qualified_name: string;
  label: string;
  size: number;
  language: string;
  file_path?: string;
  line_start?: number;
  line_end?: number;
  position?: Position3D;
  metadata: NodeMetadata;
}

export interface EdgeMetadata {
  line_number?: number;
  is_conditional?: boolean;
  is_dev_dependency?: boolean;
  version_constraint?: string;
}

export interface UnifiedEdge {
  id: string;
  source: string;
  target: string;
  edge_type: UnifiedEdgeType;
  weight: number;
  label?: string;
  detail?: string;
  bidirectional: boolean;
  metadata: EdgeMetadata;
}

export interface GraphMetadata {
  project_name: string;
  root_path: string;
  language: string;
  total_files: number;
  total_lines?: number;
  scanned_at?: string;
  parser_version: string;
}

export interface UnifiedGraph {
  nodes: UnifiedNode[];
  edges: UnifiedEdge[];
  metadata: GraphMetadata;
}

export interface DetectionResult {
  project_type: ProjectType;
  confidence: number;
  parser_id: string;
  marker_files_found: string[];
  is_multi_language: boolean;
  secondary_types: [ProjectType, number][];
}

export interface ParserInfo {
  id: string;
  display_name: string;
  description: string;
  version: string;
  file_extensions: string[];
  marker_files: string[];
  marker_dirs: string[];
  project_type: ProjectType;
  primary_color: string;
  is_available: boolean;
}

export interface SourceFile {
  name: string;
  path: string;
  absolute_path: string;
  extension: string;
  size_bytes: number;
  hash?: string;
  modified_at?: string;
}
