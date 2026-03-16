import { create } from "zustand";
import { api } from "../lib/api";

interface PatternsState {
  files: string[];
  selected: string | null;
  content: string;
  savedContent: string;
  dirty: boolean;

  fetchFiles: () => Promise<void>;
  selectFile: (name: string) => Promise<void>;
  setContent: (content: string) => void;
  save: () => Promise<void>;
  createFile: (name: string) => Promise<void>;
  deleteFile: (name: string) => Promise<void>;
}

export const usePatterns = create<PatternsState>((set, get) => ({
  files: [],
  selected: null,
  content: "",
  savedContent: "",
  dirty: false,

  fetchFiles: async () => {
    const files = await api.get<string[]>("/patterns/files");
    set({ files });
  },

  selectFile: async (name) => {
    const content = await api.getText(`/patterns/files/${encodeURIComponent(name)}`);
    set({ selected: name, content, savedContent: content, dirty: false });
  },

  setContent: (content) => {
    set((s) => ({ content, dirty: content !== s.savedContent }));
  },

  save: async () => {
    const { selected, content } = get();
    if (!selected) return;
    await api.put(`/patterns/files/${encodeURIComponent(selected)}`, content);
    set({ savedContent: content, dirty: false });
  },

  createFile: async (name) => {
    await api.post("/patterns/files", { name });
    await get().fetchFiles();
  },

  deleteFile: async (name) => {
    await api.del(`/patterns/files/${encodeURIComponent(name)}`);
    const { selected } = get();
    if (selected === name) set({ selected: null, content: "", savedContent: "", dirty: false });
    await get().fetchFiles();
  },
}));
