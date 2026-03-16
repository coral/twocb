import { create } from "zustand";

export type ConnectionStatus = "disconnected" | "connecting" | "connected" | "error";

interface ConnectionState {
  host: string;
  port: number;
  status: ConnectionStatus;
  error: string | null;

  setHostPort: (host: string, port: number) => void;
  setStatus: (status: ConnectionStatus, error?: string) => void;
  baseUrl: () => string;
  wsUrl: () => string;
}

const saved = (() => {
  try {
    const raw = localStorage.getItem("twocb-connection");
    if (raw) return JSON.parse(raw);
  } catch {}
  return null;
})();

export const useConnection = create<ConnectionState>((set, get) => ({
  host: saved?.host ?? "localhost",
  port: saved?.port ?? 3030,
  status: "disconnected",
  error: null,

  setHostPort: (host, port) => {
    localStorage.setItem("twocb-connection", JSON.stringify({ host, port }));
    set({ host, port });
  },

  setStatus: (status, error) => set({ status, error: error ?? null }),

  baseUrl: () => {
    const { host, port } = get();
    return `http://${host}:${port}`;
  },

  wsUrl: () => {
    const { host, port } = get();
    return `ws://${host}:${port}`;
  },
}));
