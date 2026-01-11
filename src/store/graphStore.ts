import { create } from "zustand";
import type { UnifiedGraph, UnifiedNode, UnifiedEdge } from "@/types/unified";

interface GraphState {
  // Data
  graph: UnifiedGraph | null;
  selectedNode: UnifiedNode | null;
  hoveredNode: UnifiedNode | null;

  // Filters
  visibleNodeTypes: Set<string>;
  visibleEdgeTypes: Set<string>;
  searchQuery: string;

  // Actions
  setGraph: (graph: UnifiedGraph) => void;
  clearGraph: () => void;
  selectNode: (node: UnifiedNode | null) => void;
  hoverNode: (node: UnifiedNode | null) => void;
  setSearchQuery: (query: string) => void;
  toggleNodeType: (type: string) => void;
  toggleEdgeType: (type: string) => void;

  // Computed
  filteredNodes: () => UnifiedNode[];
  filteredEdges: () => UnifiedEdge[];
}

export const useGraphStore = create<GraphState>((set, get) => ({
  // Initial state
  graph: null,
  selectedNode: null,
  hoveredNode: null,
  visibleNodeTypes: new Set(),
  visibleEdgeTypes: new Set(),
  searchQuery: "",

  // Actions
  setGraph: (graph) => {
    // Initialize visible types from graph
    const nodeTypes = new Set(
      graph.nodes.map((n) =>
        typeof n.node_type === "string" ? n.node_type : "custom"
      )
    );
    const edgeTypes = new Set(
      graph.edges.map((e) =>
        typeof e.edge_type === "string" ? e.edge_type : "custom"
      )
    );

    set({
      graph,
      visibleNodeTypes: nodeTypes,
      visibleEdgeTypes: edgeTypes,
    });
  },

  clearGraph: () =>
    set({
      graph: null,
      selectedNode: null,
      hoveredNode: null,
    }),

  selectNode: (node) => set({ selectedNode: node }),
  hoverNode: (node) => set({ hoveredNode: node }),
  setSearchQuery: (query) => set({ searchQuery: query }),

  toggleNodeType: (type) => {
    const current = get().visibleNodeTypes;
    const newSet = new Set(current);
    if (newSet.has(type)) {
      newSet.delete(type);
    } else {
      newSet.add(type);
    }
    set({ visibleNodeTypes: newSet });
  },

  toggleEdgeType: (type) => {
    const current = get().visibleEdgeTypes;
    const newSet = new Set(current);
    if (newSet.has(type)) {
      newSet.delete(type);
    } else {
      newSet.add(type);
    }
    set({ visibleEdgeTypes: newSet });
  },

  // Computed
  filteredNodes: () => {
    const { graph, visibleNodeTypes, searchQuery } = get();
    if (!graph) return [];

    return graph.nodes.filter((node) => {
      const typeStr =
        typeof node.node_type === "string" ? node.node_type : "custom";
      if (!visibleNodeTypes.has(typeStr)) return false;

      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        return (
          node.name.toLowerCase().includes(query) ||
          node.qualified_name.toLowerCase().includes(query) ||
          (node.file_path?.toLowerCase().includes(query) ?? false)
        );
      }

      return true;
    });
  },

  filteredEdges: () => {
    const { graph, visibleEdgeTypes } = get();
    if (!graph) return [];

    const filteredNodeIds = new Set(get().filteredNodes().map((n) => n.id));

    return graph.edges.filter((edge) => {
      const typeStr =
        typeof edge.edge_type === "string" ? edge.edge_type : "custom";
      if (!visibleEdgeTypes.has(typeStr)) return false;

      // Only show edges where both nodes are visible
      return (
        filteredNodeIds.has(edge.source) && filteredNodeIds.has(edge.target)
      );
    });
  },
}));
