import { FolderOpen, Filter, Layout, Info, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { useProjectStore } from "@/store/projectStore";
import { useGraphStore } from "@/store/graphStore";

interface SidebarProps {
  className?: string;
  onOpenProject?: () => void;
}

export function Sidebar({ className, onOpenProject }: SidebarProps) {
  const { projectPath, detection, scannedFiles, isScanning } = useProjectStore();
  const { graph, visibleNodeTypes, toggleNodeType } = useGraphStore();

  const nodeTypeCounts = graph?.nodes.reduce(
    (acc, node) => {
      const type =
        typeof node.node_type === "string" ? node.node_type : "custom";
      acc[type] = (acc[type] || 0) + 1;
      return acc;
    },
    {} as Record<string, number>
  );

  const getParserColor = (parserId: string) => {
    const colors: Record<string, string> = {
      delphi: "#E31D1D",
      laravel: "#FF2D20",
      nodejs: "#339933",
      php: "#777BB4",
    };
    return colors[parserId] || "#6B7280";
  };

  return (
    <aside
      className={cn(
        "w-72 border-r bg-sidebar flex flex-col",
        className
      )}
    >
      {/* Project Section */}
      <div className="p-4">
        <div className="flex items-center gap-2 mb-3">
          <FolderOpen className="w-4 h-4 text-sidebar-foreground" />
          <span className="text-sm font-medium text-sidebar-foreground">Project</span>
        </div>

        {projectPath ? (
          <div className="space-y-3">
            <p className="text-xs text-muted-foreground truncate" title={projectPath}>
              {projectPath}
            </p>
            {detection && (
              <div className="flex items-center gap-2">
                <div
                  className="w-3 h-3 rounded-full"
                  style={{ backgroundColor: getParserColor(detection.parser_id) }}
                />
                <span className="text-sm text-sidebar-foreground capitalize">
                  {detection.parser_id}
                </span>
                <Badge variant="secondary" className="text-xs">
                  {Math.round(detection.confidence * 100)}%
                </Badge>
              </div>
            )}
            <p className="text-xs text-muted-foreground">
              {scannedFiles.length} files scanned
            </p>
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={onOpenProject}
            >
              Change Project
            </Button>
          </div>
        ) : (
          <Button
            className="w-full"
            onClick={onOpenProject}
            disabled={isScanning}
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
        )}
      </div>

      <Separator />

      {/* Filters Section */}
      <ScrollArea className="flex-1">
        <div className="p-4">
          <div className="flex items-center gap-2 mb-3">
            <Filter className="w-4 h-4 text-sidebar-foreground" />
            <span className="text-sm font-medium text-sidebar-foreground">Filters</span>
          </div>

          {nodeTypeCounts && Object.keys(nodeTypeCounts).length > 0 ? (
            <div className="space-y-2">
              {Object.entries(nodeTypeCounts)
                .sort((a, b) => b[1] - a[1])
                .map(([type, count]) => (
                  <div
                    key={type}
                    className="flex items-center space-x-2 py-1"
                  >
                    <Checkbox
                      id={`filter-${type}`}
                      checked={visibleNodeTypes.has(type)}
                      onCheckedChange={() => toggleNodeType(type)}
                    />
                    <Label
                      htmlFor={`filter-${type}`}
                      className="flex-1 text-sm capitalize cursor-pointer text-sidebar-foreground"
                    >
                      {type.replace(/_/g, " ")}
                    </Label>
                    <Badge variant="outline" className="text-xs">
                      {count}
                    </Badge>
                  </div>
                ))}
            </div>
          ) : (
            <p className="text-xs text-muted-foreground">
              Open a project to see filters
            </p>
          )}
        </div>

        <Separator />

        {/* Layout Section */}
        <div className="p-4">
          <div className="flex items-center gap-2 mb-3">
            <Layout className="w-4 h-4 text-sidebar-foreground" />
            <span className="text-sm font-medium text-sidebar-foreground">Layout</span>
          </div>

          <div className="space-y-2">
            {["Force-directed", "Hierarchical", "Radial"].map((layout) => (
              <div key={layout} className="flex items-center space-x-2">
                <input
                  type="radio"
                  id={`layout-${layout}`}
                  name="layout"
                  defaultChecked={layout === "Force-directed"}
                  className="text-primary"
                />
                <Label
                  htmlFor={`layout-${layout}`}
                  className="text-sm cursor-pointer text-sidebar-foreground"
                >
                  {layout}
                </Label>
              </div>
            ))}
          </div>
        </div>
      </ScrollArea>

      <Separator />

      {/* Info Section */}
      <div className="p-4">
        <div className="flex items-center gap-2 mb-2">
          <Info className="w-4 h-4 text-sidebar-foreground" />
          <span className="text-sm font-medium text-sidebar-foreground">Stats</span>
        </div>
        <div className="grid grid-cols-2 gap-2 text-xs text-muted-foreground">
          <div>Nodes: <span className="text-sidebar-foreground font-medium">{graph?.nodes.length ?? 0}</span></div>
          <div>Edges: <span className="text-sidebar-foreground font-medium">{graph?.edges.length ?? 0}</span></div>
        </div>
      </div>
    </aside>
  );
}
