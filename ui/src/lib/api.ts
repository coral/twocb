import { useConnection } from "../stores/connection";

function base(): string {
  return useConnection.getState().baseUrl();
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${base()}${path}`, init);
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.json();
}

async function requestText(path: string): Promise<string> {
  const res = await fetch(`${base()}${path}`);
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.text();
}

export const api = {
  get: <T>(path: string) => request<T>(path),

  getText: (path: string) => requestText(path),

  post: <T = unknown>(path: string, body?: unknown) =>
    request<T>(path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: body != null ? JSON.stringify(body) : undefined,
    }),

  put: async (path: string, body: string) => {
    const res = await fetch(`${base()}${path}`, {
      method: "PUT",
      headers: { "Content-Type": "text/javascript" },
      body,
    });
    if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  },

  del: async (path: string) => {
    const res = await fetch(`${base()}${path}`, { method: "DELETE" });
    if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  },
};
