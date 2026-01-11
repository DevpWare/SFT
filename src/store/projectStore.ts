import { create } from "zustand";
import type { DetectionResult, ParserInfo, SourceFile } from "@/types/unified";

interface ProjectState {
  // Project info
  projectPath: string | null;
  detection: DetectionResult | null;
  availableParsers: ParserInfo[];
  selectedParserId: string | null;

  // Scan state
  isScanning: boolean;
  scanProgress: number;
  scannedFiles: SourceFile[];

  // Actions
  setProjectPath: (path: string) => void;
  setDetection: (detection: DetectionResult) => void;
  setParsers: (parsers: ParserInfo[]) => void;
  selectParser: (parserId: string) => void;
  setScanning: (isScanning: boolean) => void;
  setScanProgress: (progress: number) => void;
  setScannedFiles: (files: SourceFile[]) => void;
  reset: () => void;
}

export const useProjectStore = create<ProjectState>((set) => ({
  // Initial state
  projectPath: null,
  detection: null,
  availableParsers: [],
  selectedParserId: null,
  isScanning: false,
  scanProgress: 0,
  scannedFiles: [],

  // Actions
  setProjectPath: (path) => set({ projectPath: path }),
  setDetection: (detection) =>
    set({
      detection,
      selectedParserId: detection.parser_id,
    }),
  setParsers: (parsers) => set({ availableParsers: parsers }),
  selectParser: (parserId) => set({ selectedParserId: parserId }),
  setScanning: (isScanning) => set({ isScanning }),
  setScanProgress: (progress) => set({ scanProgress: progress }),
  setScannedFiles: (files) => set({ scannedFiles: files }),
  reset: () =>
    set({
      projectPath: null,
      detection: null,
      selectedParserId: null,
      isScanning: false,
      scanProgress: 0,
      scannedFiles: [],
    }),
}));
