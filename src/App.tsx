import { useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Header } from "@/components/layout/Header";
import { Sidebar } from "@/components/layout/Sidebar";
import { StatusBar } from "@/components/layout/StatusBar";
import { Graph3D } from "@/components/graph/Graph3D";
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

  // Load available parsers on mount
  useEffect(() => {
    tauriCommands.listParsers().then(setParsers).catch(console.error);
  }, [setParsers]);

  const handleOpenProject = async () => {
    try {
      console.log("Opening folder dialog...");
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Project Directory",
      });

      console.log("Selected:", selected);

      if (selected && typeof selected === "string") {
        setProjectPath(selected);
        setScanning(true);

        // Detect project type
        console.log("Detecting project type...");
        const detection = await tauriCommands.detectProjectType(selected);
        console.log("Detection result:", detection);
        setDetection(detection);

        // Scan directory
        console.log("Scanning directory...");
        const files = await tauriCommands.scanDirectory(
          selected,
          detection.parser_id
        );
        console.log("Scanned files:", files.length);
        setScannedFiles(files);

        // Create a simple graph from scanned files for now
        // Use deterministic positions based on index for stability
        const simpleGraph = {
          nodes: files.slice(0, 200).map((file, index) => {
            // Create a grid-like layout
            const cols = Math.ceil(Math.sqrt(files.length));
            const row = Math.floor(index / cols);
            const col = index % cols;

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
                x: (col - cols / 2) * 3,
                y: (row - Math.ceil(files.length / cols) / 2) * 3,
                z: (Math.random() - 0.5) * 5,
              },
              metadata: {
                size_bytes: file.size_bytes,
              },
            };
          }),
          edges: [],
          metadata: {
            project_name: selected.split("/").pop() ?? "Project",
            root_path: selected,
            language: detection.parser_id,
            total_files: files.length,
            parser_version: "1.0.0",
          },
        };

        console.log("Setting graph with", simpleGraph.nodes.length, "nodes");
        setGraph(simpleGraph);
        setScanning(false);
      }
    } catch (error) {
      console.error("Error opening project:", error);
      setScanning(false);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-background">
      <Header />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar onOpenProject={handleOpenProject} />
        <Graph3D />
      </div>
      <StatusBar />
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

export default App;
