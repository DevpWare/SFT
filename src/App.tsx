import { useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Graph3D } from "@/components/graph/Graph3D";
import { FloatingUI } from "@/components/layout/FloatingUI";
import { useProjectStore } from "@/store/projectStore";
import { useGraphStore } from "@/store/graphStore";
import tauriCommands from "@/lib/tauri";

function App() {
  const {
    setProjectPath,
    setDetection,
    setParsers,
    setScanning,
    setScannedFiles,
  } = useProjectStore();

  const { setGraph } = useGraphStore();

  useEffect(() => {
    tauriCommands.listParsers().then(setParsers).catch(console.error);
  }, [setParsers]);

  useEffect(() => {
    document.documentElement.classList.add("dark");
  }, []);

  const handleOpenProject = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Project Directory",
      });

      if (selected && typeof selected === "string") {
        setProjectPath(selected);
        setScanning(true);

        const detection = await tauriCommands.detectProjectType(selected);
        setDetection(detection);

        const files = await tauriCommands.scanDirectory(
          selected,
          detection.parser_id
        );
        setScannedFiles(files);

        const nodeCount = Math.min(files.length, 200);
        const simpleGraph = {
          nodes: files.slice(0, 200).map((file, index) => {
            const phi = Math.acos(-1 + (2 * index) / nodeCount);
            const theta = Math.sqrt(nodeCount * Math.PI) * phi;
            const radius = 30 + Math.random() * 10;

            return {
              id: `file-${index}`,
              node_type: getNodeTypeFromExtension(file.extension),
              name: file.name,
              qualified_name: file.path,
              label: file.name,
              size: Math.min(10, Math.max(3, Math.floor(file.size_bytes / 3000) + 3)),
              language: detection.parser_id,
              file_path: file.path,
              position: {
                x: radius * Math.cos(theta) * Math.sin(phi),
                y: radius * Math.sin(theta) * Math.sin(phi),
                z: radius * Math.cos(phi),
              },
              metadata: {
                size_bytes: file.size_bytes,
              },
            };
          }),
          edges: generateEdges(files.slice(0, 200)),
          metadata: {
            project_name: selected.split("/").pop() ?? "Project",
            root_path: selected,
            language: detection.parser_id,
            total_files: files.length,
            parser_version: "1.0.0",
          },
        };

        setGraph(simpleGraph);
        setScanning(false);
      }
    } catch (error) {
      console.error("Error opening project:", error);
      setScanning(false);
    }
  };

  return (
    <div className="relative w-screen h-screen overflow-hidden bg-black">
      <Graph3D className="absolute inset-0" />
      <FloatingUI onOpenProject={handleOpenProject} />
    </div>
  );
}

function getNodeTypeFromExtension(ext: string): "module" | "form" | "source_file" | "component" | "controller" | "model" | "view" {
  const mapping: Record<string, "module" | "form" | "source_file" | "component" | "controller" | "model" | "view"> = {
    pas: "module",
    dfm: "form",
    fmx: "form",
    dpr: "source_file",
    php: "source_file",
    js: "module",
    ts: "module",
    tsx: "component",
    jsx: "component",
  };
  return mapping[ext.toLowerCase()] ?? "source_file";
}

function generateEdges(files: Array<{ path: string; name: string }>) {
  const edges: Array<{
    id: string;
    source: string;
    target: string;
    edge_type: "references" | { custom: string };
    weight: number;
    bidirectional: boolean;
    metadata: Record<string, unknown>;
  }> = [];

  for (let i = 0; i < files.length; i++) {
    const dir1 = files[i].path.split("/").slice(0, -1).join("/");
    for (let j = i + 1; j < Math.min(i + 10, files.length); j++) {
      const dir2 = files[j].path.split("/").slice(0, -1).join("/");
      if (dir1 === dir2) {
        edges.push({
          id: `edge-${i}-${j}`,
          source: `file-${i}`,
          target: `file-${j}`,
          edge_type: { custom: "same_directory" },
          weight: 1,
          bidirectional: true,
          metadata: {},
        });
      }
    }
  }

  return edges;
}

export default App;
