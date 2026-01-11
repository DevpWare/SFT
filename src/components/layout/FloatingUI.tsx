import { useState } from "react";
import { Search, FolderOpen, X, Loader2, ChevronLeft, ExternalLink, Settings, RotateCcw, Eye, EyeOff } from "lucide-react";
import LogoIcon from "@/assets/logo-icon.svg";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { useProjectStore } from "@/store/projectStore";
import { useGraphStore } from "@/store/graphStore";
import { useSettingsStore, DEFAULT_NODE_COLORS } from "@/store/settingsStore";

interface FloatingUIProps {
  onOpenProject: () => void;
}

export function FloatingUI({ onOpenProject }: FloatingUIProps) {
  const { projectPath, detection, scannedFiles, isScanning } = useProjectStore();
  const {
    graph,
    selectedNode,
    selectNode,
    setSearchQuery,
    visibleNodeTypes,
    toggleNodeType,
    filteredNodes,
    filteredEdges
  } = useGraphStore();

  const {
    nodeColors,
    showLegend,
    setNodeColor,
    resetNodeColors,
    toggleLegend,
    rotationSpeed,
    setRotationSpeed,
    nodeSize,
    setNodeSize
  } = useSettingsStore();

  const [showSettings, setShowSettings] = useState(false);
  const [connectionSearch, setConnectionSearch] = useState("");

  const nodeTypeCounts = graph?.nodes.reduce(
    (acc, node) => {
      const type = typeof node.node_type === "string" ? node.node_type : "custom";
      acc[type] = (acc[type] || 0) + 1;
      return acc;
    },
    {} as Record<string, number>
  );

  const connectedEdges = selectedNode
    ? graph?.edges.filter(
        (e) => e.source === selectedNode.id || e.target === selectedNode.id
      ) ?? []
    : [];

  const filteredConnections = connectedEdges.filter((edge) => {
    if (!connectionSearch) return true;
    const otherId = edge.source === selectedNode?.id ? edge.target : edge.source;
    const otherNode = graph?.nodes.find((n) => n.id === otherId);
    return otherNode?.name.toLowerCase().includes(connectionSearch.toLowerCase());
  });

  return (
    <div className="absolute inset-0 pointer-events-none">
      {/* Logo - Top Left */}
      <div className="absolute top-6 left-6 pointer-events-auto">
        <div className="flex items-center gap-3">
          <img src={LogoIcon} alt="DevWare" className="w-12 h-12" />
          <div>
            <h1 className="text-lg font-semibold text-white">DevWare</h1>
            <p className="text-xs text-white/50">Dependency Analyzer</p>
          </div>
        </div>
      </div>

      {/* Search + Settings - Top Right */}
      <div className="absolute top-6 right-6 pointer-events-auto flex items-center gap-2">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-white/50" />
          <Input
            type="text"
            placeholder="Search nodes..."
            className="pl-10 w-64 bg-black/40 backdrop-blur-md border-white/10 text-white placeholder:text-white/40 focus:border-primary/50"
            onChange={(e) => setSearchQuery(e.target.value)}
          />
          <kbd className="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-white/30 bg-white/5 px-1.5 py-0.5 rounded">
            Ctrl+K
          </kbd>
        </div>
        <Button
          variant="ghost"
          size="icon"
          onClick={() => setShowSettings(!showSettings)}
          className="text-white/60 hover:text-white hover:bg-white/10"
        >
          <Settings className="w-5 h-5" />
        </Button>
      </div>

      {/* Settings Panel */}
      {showSettings && (
        <div className="absolute top-20 right-6 w-80 pointer-events-auto">
          <div className="bg-black/70 backdrop-blur-md border border-white/10 rounded-xl overflow-hidden shadow-2xl">
            <div className="p-4 border-b border-white/10 flex items-center justify-between">
              <h3 className="text-white font-medium">Settings</h3>
              <button
                onClick={() => setShowSettings(false)}
                className="text-white/40 hover:text-white"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="p-4 space-y-6">
              {/* Rotation Speed */}
              <div>
                <div className="flex items-center justify-between mb-2">
                  <label className="text-white/70 text-sm">Rotation Speed</label>
                  <span className="text-white/40 text-xs">{(rotationSpeed * 100).toFixed(0)}%</span>
                </div>
                <Slider
                  value={[rotationSpeed * 100]}
                  onValueChange={(values: number[]) => setRotationSpeed(values[0] / 100)}
                  min={0}
                  max={10}
                  step={1}
                  className="w-full"
                />
              </div>

              {/* Node Size */}
              <div>
                <div className="flex items-center justify-between mb-2">
                  <label className="text-white/70 text-sm">Node Size</label>
                  <span className="text-white/40 text-xs">{nodeSize.toFixed(1)}x</span>
                </div>
                <Slider
                  value={[nodeSize * 10]}
                  onValueChange={(values: number[]) => setNodeSize(values[0] / 10)}
                  min={5}
                  max={20}
                  step={1}
                  className="w-full"
                />
              </div>

              {/* Show Legend Toggle */}
              <div className="flex items-center justify-between">
                <label className="text-white/70 text-sm">Show Legend</label>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={toggleLegend}
                  className="text-white/60 hover:text-white"
                >
                  {showLegend ? <Eye className="w-4 h-4" /> : <EyeOff className="w-4 h-4" />}
                </Button>
              </div>

              {/* Node Colors */}
              <div>
                <div className="flex items-center justify-between mb-3">
                  <label className="text-white/70 text-sm">Node Colors</label>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={resetNodeColors}
                    className="text-white/40 hover:text-white text-xs h-6 px-2"
                  >
                    <RotateCcw className="w-3 h-3 mr-1" />
                    Reset
                  </Button>
                </div>
                <ScrollArea className="h-48">
                  <div className="space-y-2 pr-2">
                    {Object.entries(nodeColors).map(([type, color]) => (
                      <div key={type} className="flex items-center gap-3">
                        <input
                          type="color"
                          value={color}
                          onChange={(e) => setNodeColor(type, e.target.value)}
                          className="w-8 h-8 rounded cursor-pointer bg-transparent border border-white/20"
                        />
                        <span className="text-white/70 text-sm capitalize flex-1">
                          {type.replace(/_/g, " ")}
                        </span>
                        {color !== DEFAULT_NODE_COLORS[type] && (
                          <button
                            onClick={() => setNodeColor(type, DEFAULT_NODE_COLORS[type])}
                            className="text-white/30 hover:text-white/60 text-xs"
                          >
                            Reset
                          </button>
                        )}
                      </div>
                    ))}
                  </div>
                </ScrollArea>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Legend - Bottom Right */}
      {showLegend && graph && nodeTypeCounts && Object.keys(nodeTypeCounts).length > 0 && (
        <div className="absolute bottom-6 right-6 pointer-events-auto">
          <div className="bg-black/50 backdrop-blur-md border border-white/10 rounded-xl p-4">
            <h3 className="text-white/70 text-xs font-medium mb-3 uppercase tracking-wider">
              Legend
            </h3>
            <div className="space-y-2">
              {Object.entries(nodeTypeCounts)
                .sort((a, b) => b[1] - a[1])
                .map(([type, count]) => (
                  <div key={type} className="flex items-center gap-2">
                    <div
                      className="w-3 h-3 rounded-full"
                      style={{ backgroundColor: nodeColors[type] || nodeColors.custom }}
                    />
                    <span className="text-white/70 text-xs capitalize flex-1">
                      {type.replace(/_/g, " ")}
                    </span>
                    <span className="text-white/30 text-xs">{count}</span>
                  </div>
                ))}
            </div>
          </div>
        </div>
      )}

      {/* Left Panel - Stats + Filters */}
      <div className="absolute bottom-6 left-6 pointer-events-auto flex flex-col gap-4">
        {/* Filters */}
        {graph && nodeTypeCounts && Object.keys(nodeTypeCounts).length > 0 && (
          <div className="bg-black/50 backdrop-blur-md border border-white/10 rounded-xl p-4 w-64">
            <h3 className="text-white/70 text-xs font-medium mb-3 uppercase tracking-wider">
              Filter by Type
            </h3>
            <div className="space-y-2">
              {Object.entries(nodeTypeCounts)
                .sort((a, b) => b[1] - a[1])
                .slice(0, 8)
                .map(([type, count]) => (
                  <div key={type} className="flex items-center gap-2">
                    <Checkbox
                      id={`filter-${type}`}
                      checked={visibleNodeTypes.has(type)}
                      onCheckedChange={() => toggleNodeType(type)}
                      className="border-white/30 data-[state=checked]:bg-primary"
                    />
                    <div
                      className="w-2 h-2 rounded-full"
                      style={{ backgroundColor: nodeColors[type] || nodeColors.custom }}
                    />
                    <Label
                      htmlFor={`filter-${type}`}
                      className="flex-1 text-sm text-white/70 capitalize cursor-pointer"
                    >
                      {type.replace(/_/g, " ")}
                    </Label>
                    <span className="text-white/30 text-xs">{count}</span>
                  </div>
                ))}
            </div>
          </div>
        )}

        {/* Stats Panel */}
        <div className="bg-black/50 backdrop-blur-md border border-white/10 rounded-xl p-4 w-64">
          {!projectPath ? (
            <div className="space-y-4">
              <div>
                <h2 className="text-white font-medium mb-1">No project loaded</h2>
                <p className="text-white/50 text-sm">Select a project folder to analyze</p>
              </div>
              <Button
                onClick={onOpenProject}
                disabled={isScanning}
                className="w-full"
              >
                {isScanning ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    Scanning...
                  </>
                ) : (
                  <>
                    <FolderOpen className="w-4 h-4 mr-2" />
                    Open Project
                  </>
                )}
              </Button>
            </div>
          ) : (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-white font-medium truncate max-w-[150px]">
                    {graph?.metadata?.project_name ?? "Project"}
                  </h2>
                  {detection && (
                    <div className="flex items-center gap-2 mt-1">
                      <Badge variant="secondary" className="text-xs capitalize">
                        {detection.parser_id}
                      </Badge>
                      <span className="text-white/40 text-xs">
                        {Math.round(detection.confidence * 100)}%
                      </span>
                    </div>
                  )}
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={onOpenProject}
                  className="text-white/60 hover:text-white text-xs"
                >
                  Change
                </Button>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="bg-white/5 rounded-lg p-3">
                  <p className="text-white/50 text-xs mb-1">NODES</p>
                  <p className="text-xl font-semibold text-white">
                    {filteredNodes().length.toLocaleString()}
                  </p>
                </div>
                <div className="bg-white/5 rounded-lg p-3">
                  <p className="text-white/50 text-xs mb-1">EDGES</p>
                  <p className="text-xl font-semibold text-white">
                    {filteredEdges().length.toLocaleString()}
                  </p>
                </div>
              </div>

              <div className="text-white/40 text-xs">
                {scannedFiles.length} files scanned
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Node Detail Panel - Right Side */}
      {selectedNode && !showSettings && (
        <div className="absolute top-20 right-6 w-80 pointer-events-auto">
          <div className="bg-black/70 backdrop-blur-md border border-white/10 rounded-xl overflow-hidden shadow-2xl">
            <div className="p-4 border-b border-white/10 flex items-center justify-between">
              <button
                onClick={() => selectNode(null)}
                className="flex items-center gap-1 text-white/60 hover:text-white text-sm transition-colors"
              >
                <ChevronLeft className="w-4 h-4" />
                Back
              </button>
              <button
                onClick={() => selectNode(null)}
                className="text-white/40 hover:text-white transition-colors"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="p-4 space-y-4">
              <div>
                <h2 className="text-white font-medium text-lg break-all">
                  {selectedNode.name}
                </h2>
                <div className="flex items-center gap-2 mt-2">
                  <div
                    className="w-3 h-3 rounded-full"
                    style={{
                      backgroundColor:
                        nodeColors[
                          typeof selectedNode.node_type === "string"
                            ? selectedNode.node_type
                            : "custom"
                        ] || nodeColors.custom,
                    }}
                  />
                  <Badge variant="outline" className="text-xs capitalize border-white/20 text-white/60">
                    {typeof selectedNode.node_type === "string"
                      ? selectedNode.node_type.replace(/_/g, " ")
                      : "custom"}
                  </Badge>
                </div>
              </div>

              {selectedNode.file_path && (
                <div>
                  <p className="text-white/40 text-xs mb-1">PATH</p>
                  <p className="text-white/70 text-sm break-all font-mono bg-white/5 p-2 rounded">
                    {selectedNode.file_path}
                  </p>
                </div>
              )}

              {selectedNode.metadata?.size_bytes && (
                <div>
                  <p className="text-white/40 text-xs mb-1">SIZE</p>
                  <p className="text-white/70 text-sm">
                    {(selectedNode.metadata.size_bytes / 1024).toFixed(1)} KB
                  </p>
                </div>
              )}

              {connectedEdges.length > 0 && (
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <p className="text-white/40 text-xs">
                      CONNECTIONS ({connectedEdges.length})
                    </p>
                  </div>

                  {/* Search in connections */}
                  <div className="relative mb-2">
                    <Search className="absolute left-2 top-1/2 -translate-y-1/2 w-3 h-3 text-white/30" />
                    <Input
                      type="text"
                      placeholder="Search connections..."
                      value={connectionSearch}
                      onChange={(e) => setConnectionSearch(e.target.value)}
                      className="pl-7 h-8 text-xs bg-white/5 border-white/10 text-white placeholder:text-white/30"
                    />
                    {connectionSearch && (
                      <button
                        onClick={() => setConnectionSearch("")}
                        className="absolute right-2 top-1/2 -translate-y-1/2 text-white/30 hover:text-white/60"
                      >
                        <X className="w-3 h-3" />
                      </button>
                    )}
                  </div>

                  <ScrollArea className="h-40">
                    <div className="space-y-1 pr-2">
                      {filteredConnections.length > 0 ? (
                        filteredConnections.slice(0, 20).map((edge) => {
                          const otherId =
                            edge.source === selectedNode.id
                              ? edge.target
                              : edge.source;
                          const otherNode = graph?.nodes.find(
                            (n) => n.id === otherId
                          );
                          const otherType =
                            typeof otherNode?.node_type === "string"
                              ? otherNode.node_type
                              : "custom";
                          return (
                            <button
                              key={edge.id}
                              onClick={() => otherNode && selectNode(otherNode)}
                              className="w-full text-left px-3 py-2 rounded bg-white/5 hover:bg-white/10 transition-colors flex items-center gap-2 group"
                            >
                              <div
                                className="w-2 h-2 rounded-full flex-shrink-0"
                                style={{
                                  backgroundColor:
                                    nodeColors[otherType] || nodeColors.custom,
                                }}
                              />
                              <p className="text-white/70 text-sm truncate flex-1">
                                {otherNode?.name ?? otherId}
                              </p>
                              <ExternalLink className="w-3 h-3 text-white/30 group-hover:text-white/60 flex-shrink-0" />
                            </button>
                          );
                        })
                      ) : (
                        <p className="text-white/30 text-xs text-center py-4">
                          No connections found
                        </p>
                      )}
                    </div>
                  </ScrollArea>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
