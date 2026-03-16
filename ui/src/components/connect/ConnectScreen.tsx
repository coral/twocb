import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useConnection } from "../../stores/connection";
import { useLayers } from "../../stores/layers";
import { usePixels } from "../../stores/pixels";
import { useSettings } from "../../stores/settings";
import { connectStateWs } from "../../lib/ws";
import { connectPixelStream } from "../../lib/rtc";

export default function ConnectScreen() {
  const { host, port, setHostPort, setStatus, status, error } = useConnection();
  const [h, setH] = useState(host);
  const [p, setP] = useState(String(port));
  const navigate = useNavigate();

  const connect = async () => {
    setStatus("connecting");
    setHostPort(h, Number(p));

    try {
      // Validate connection with a ping
      const res = await fetch(`http://${h}:${p}/layers`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);

      setStatus("connected");

      // Bootstrap stores
      useLayers.getState().fetchLayers();
      useLayers.getState().fetchOrder();
      usePixels.getState().fetchMapping();
      useSettings.getState().fetchConfig();

      // Connect WebSocket + WebRTC
      connectStateWs();
      connectPixelStream();

      navigate("/app");
    } catch (e) {
      setStatus("error", e instanceof Error ? e.message : "Connection failed");
    }
  };

  return (
    <div className="flex h-screen items-center justify-center">
      <div className="w-80 rounded-lg bg-surface-1 p-6 border border-border">
        <h1 className="text-xl font-semibold mb-1">twocb</h1>
        <p className="text-neutral-500 text-sm mb-6">Connect to backend</p>

        <label className="block text-xs text-neutral-400 mb-1">Host</label>
        <input
          className="w-full rounded bg-surface-2 border border-border px-3 py-2 text-sm mb-3 outline-none focus:border-accent"
          value={h}
          onChange={(e) => setH(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && connect()}
        />

        <label className="block text-xs text-neutral-400 mb-1">Port</label>
        <input
          className="w-full rounded bg-surface-2 border border-border px-3 py-2 text-sm mb-4 outline-none focus:border-accent"
          value={p}
          onChange={(e) => setP(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && connect()}
        />

        {error && (
          <p className="text-red-400 text-xs mb-3">{error}</p>
        )}

        <button
          className="w-full rounded bg-accent hover:bg-accent-hover text-white py-2 text-sm font-medium disabled:opacity-50"
          onClick={connect}
          disabled={status === "connecting"}
        >
          {status === "connecting" ? "Connecting..." : "Connect"}
        </button>
      </div>
    </div>
  );
}
