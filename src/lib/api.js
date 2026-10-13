import { invoke } from "@tauri-apps/api/core";

export const api = {
  analyzeText: (text) => invoke("analyze_text", { text }),
  analyzeFile: (path) => invoke("analyze_file", { path }),
  saveText: (path, text) => invoke("save_text", { path, text }),
};

export const LEVEL = {
  gruen: { word: "Unbedenklich", color: "var(--gruen)" },
  gelb: { word: "Vorsicht", color: "var(--gelb)" },
  rot: { word: "Nicht in die KI", color: "var(--rot)" },
};
export const groupColor = (g) => `var(--g-${g})`;
export const num = (n) => Math.round(n).toLocaleString("de-DE");
