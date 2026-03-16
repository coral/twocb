import { create } from "zustand";
import type { Config } from "../types/api";
import { api } from "../lib/api";

interface SettingsState {
  config: Config | null;
  fetchConfig: () => Promise<void>;
}

export const useSettings = create<SettingsState>((set) => ({
  config: null,

  fetchConfig: async () => {
    const config = await api.get<Config>("/config");
    set({ config });
  },
}));
