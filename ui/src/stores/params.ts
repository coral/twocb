import { create } from "zustand";
import type { PatternParam, ParamValue } from "../types/api";
import { api } from "../lib/api";

interface ParamsState {
  params: PatternParam[];
  patternName: string | null;

  fetchParams: (patternName: string) => Promise<void>;
  setParam: (paramName: string, value: ParamValue) => Promise<void>;
  clear: () => void;
}

export const useParams = create<ParamsState>((set, get) => ({
  params: [],
  patternName: null,

  fetchParams: async (patternName) => {
    const params = await api.get<PatternParam[]>(
      `/patterns/${encodeURIComponent(patternName)}/params`,
    );
    set({ params, patternName });
  },

  setParam: async (paramName, value) => {
    const { patternName } = get();
    if (!patternName) return;
    await api.post(
      `/patterns/${encodeURIComponent(patternName)}/params/${encodeURIComponent(paramName)}`,
      value,
    );
    // Optimistic update
    set((s) => ({
      params: s.params.map((p) => (p.name === paramName ? { ...p, value } : p)),
    }));
  },

  clear: () => set({ params: [], patternName: null }),
}));
