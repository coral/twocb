import { create } from "zustand";
import type { Pixel } from "../types/api";
import { api } from "../lib/api";

interface PixelsState {
  mapping: Pixel[];
  /** Flat Float32Array of RGB values for Three.js color buffer (length = pixelCount * 3) */
  colors: Float32Array;
  pixelCount: number;

  fetchMapping: () => Promise<void>;
  updateColors: (rgbU8: Uint8Array) => void;
}

export const usePixels = create<PixelsState>((set, get) => ({
  mapping: [],
  colors: new Float32Array(0),
  pixelCount: 0,

  fetchMapping: async () => {
    const mapping = await api.get<Pixel[]>("/mapping");
    const pixelCount = mapping.length;
    set({
      mapping,
      pixelCount,
      colors: new Float32Array(pixelCount * 3), // initialized to black
    });
  },

  updateColors: (rgbU8) => {
    const { colors } = get();
    const len = Math.min(rgbU8.length, colors.length);
    for (let i = 0; i < len; i++) {
      colors[i] = rgbU8[i] / 255;
    }
    // No set() — we mutate in-place to avoid re-renders.
    // Three.js reads this buffer directly via ref.
  },
}));
