// Tauri IPC wrapper

import { invoke } from "@tauri-apps/api/core";
import type {
  DetectionResult,
  ParserInfo,
  SourceFile,
  UnifiedGraph,
} from "@/types/unified";

export const tauriCommands = {
  // Detection
  detectProjectType: (path: string): Promise<DetectionResult> =>
    invoke("detect_project_type", { path }),

  // Parsers
  listParsers: (): Promise<ParserInfo[]> => invoke("list_parsers"),

  // Scanning
  scanDirectory: (
    path: string,
    parserId?: string
  ): Promise<SourceFile[]> =>
    invoke("scan_directory", { path, parserId }),

  // Placeholder for future commands
  parseProject: (
    path: string,
    parserId?: string
  ): Promise<UnifiedGraph> =>
    invoke("parse_project", { path, parserId }),
};

export default tauriCommands;
