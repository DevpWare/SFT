// Tauri IPC wrapper

import { invoke } from "@tauri-apps/api/core";
import type {
  DetectionResult,
  ParserInfo,
  SourceFile,
  UnifiedGraph,
} from "@/types/unified";

export const tauriCommands = {
  detectProjectType: (path: string): Promise<DetectionResult> =>
    invoke("detect_project_type", { path }),

  listParsers: (): Promise<ParserInfo[]> => invoke("list_parsers"),

  scanDirectory: (path: string, parserId?: string): Promise<SourceFile[]> =>
    invoke("scan_directory", { path, parserId }),

  parseProject: (path: string, parserId?: string): Promise<UnifiedGraph> =>
    invoke("parse_project", { path, parserId }),
};

export default tauriCommands;
