import { cn } from "@/lib/utils";
import { useGraphStore } from "@/store/graphStore";
import { useProjectStore } from "@/store/projectStore";
import { Separator } from "@/components/ui/separator";

interface StatusBarProps {
  className?: string;
}

export function StatusBar({ className }: StatusBarProps) {
  const { graph, selectedNode, filteredNodes, filteredEdges } = useGraphStore();
  const { isScanning, scanProgress } = useProjectStore();

  return (
    <footer
      className={cn(
        "h-8 px-4 border-t bg-card flex items-center text-xs text-muted-foreground gap-4",
        className
      )}
    >
      {isScanning ? (
        <span className="text-primary">Scanning... {Math.round(scanProgress * 100)}%</span>
      ) : (
        <>
          <span>
            Nodes: <span className="text-foreground">{filteredNodes().length}</span>
            {graph && filteredNodes().length !== graph.nodes.length && (
              <span className="opacity-60"> / {graph.nodes.length}</span>
            )}
          </span>

          <Separator orientation="vertical" className="h-4" />

          <span>
            Edges: <span className="text-foreground">{filteredEdges().length}</span>
            {graph && filteredEdges().length !== graph.edges.length && (
              <span className="opacity-60"> / {graph.edges.length}</span>
            )}
          </span>
        </>
      )}

      {selectedNode && (
        <>
          <Separator orientation="vertical" className="h-4" />
          <span>
            Selected: <span className="text-foreground font-medium">{selectedNode.name}</span>
          </span>
        </>
      )}

      <span className="ml-auto">DevpWareSoft v2.0</span>
    </footer>
  );
}
