import { create } from "zustand";
import type { Link, Order } from "../types/api";
import { api } from "../lib/api";

interface LayersState {
  links: Link[];
  order: Order[];
  loading: boolean;

  fetchLayers: () => Promise<void>;
  fetchOrder: () => Promise<void>;
  addLayer: (name: string, patternName: string, engineType: string) => Promise<void>;
  removeLayer: (name: string) => Promise<void>;
  setOpacity: (name: string, opacity: number) => Promise<void>;
}

export const useLayers = create<LayersState>((set, get) => ({
  links: [],
  order: [],
  loading: false,

  fetchLayers: async () => {
    set({ loading: true });
    try {
      const links = await api.get<Link[]>("/layers");
      set({ links, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  fetchOrder: async () => {
    try {
      const order = await api.get<Order[]>("/layers/order");
      set({ order });
    } catch {}
  },

  addLayer: async (name, patternName, engineType) => {
    await api.post("/layer", {
      name,
      steps: [{ pattern: patternName, engine_type: engineType, blendmode: "Add" }],
    });
    await get().fetchLayers();
    await get().fetchOrder();
  },

  removeLayer: async (name) => {
    await api.del(`/layer/${encodeURIComponent(name)}`);
    await get().fetchLayers();
    await get().fetchOrder();
  },

  setOpacity: async (name, opacity) => {
    await api.post("/opacity", { key: name, opacity });
    // Optimistic update
    set((s) => ({
      links: s.links.map((l) =>
        l.name === name ? { ...l, opacity } : l,
      ),
    }));
  },
}));
