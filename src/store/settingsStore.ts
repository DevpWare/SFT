import { create } from "zustand";
import { persist } from "zustand/middleware";

export const DEFAULT_NODE_COLORS: Record<string, string> = {
  module: "#1e9df1",
  form: "#e91e63",
  class: "#9c27b0",
  function: "#17bf63",
  method: "#00b87a",
  component: "#00bcd4",
  source_file: "#72767a",
  controller: "#ff2d20",
  model: "#f7b928",
  view: "#9b59b6",
  route: "#f39c12",
  interface: "#68217a",
  custom: "#6b7280",
};

interface SettingsState {
  nodeColors: Record<string, string>;
  showLegend: boolean;
  rotationSpeed: number;
  nodeSize: number;

  setNodeColor: (type: string, color: string) => void;
  resetNodeColors: () => void;
  toggleLegend: () => void;
  setRotationSpeed: (speed: number) => void;
  setNodeSize: (size: number) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      nodeColors: { ...DEFAULT_NODE_COLORS },
      showLegend: true,
      rotationSpeed: 0.03,
      nodeSize: 1,

      setNodeColor: (type, color) =>
        set((state) => ({
          nodeColors: { ...state.nodeColors, [type]: color },
        })),

      resetNodeColors: () =>
        set({ nodeColors: { ...DEFAULT_NODE_COLORS } }),

      toggleLegend: () =>
        set((state) => ({ showLegend: !state.showLegend })),

      setRotationSpeed: (speed) =>
        set({ rotationSpeed: speed }),

      setNodeSize: (size) =>
        set({ nodeSize: size }),
    }),
    {
      name: "devpwaresoft-settings",
    }
  )
);
