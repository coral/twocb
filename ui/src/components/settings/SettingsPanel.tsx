import { useEffect } from "react";
import { useSettings } from "../../stores/settings";

export default function SettingsPanel() {
  const { config, fetchConfig } = useSettings();

  useEffect(() => {
    fetchConfig();
  }, []);

  if (!config) {
    return (
      <div className="flex items-center justify-center h-full text-neutral-600 text-sm">
        Loading config...
      </div>
    );
  }

  return (
    <div className="max-w-lg mx-auto p-6 space-y-6">
      <h2 className="text-lg font-semibold">Settings</h2>

      {/* FPS */}
      <div>
        <label className="block text-xs text-neutral-400 mb-1">Frame Rate (FPS)</label>
        <input
          type="number"
          value={config.fps}
          readOnly
          className="w-32 rounded bg-surface-2 border border-border px-3 py-2 text-sm outline-none"
        />
        <p className="text-xs text-neutral-600 mt-1">Runtime FPS changes not yet implemented</p>
      </div>

      {/* Endpoints */}
      <div>
        <h3 className="text-sm font-medium mb-2">Output Endpoints</h3>
        <div className="space-y-2">
          {config.endpoints.map((ep, i) => (
            <div key={i} className="rounded bg-surface-2 border border-border p-3 text-xs space-y-1">
              <div className="flex gap-4">
                <span className="text-neutral-400">Protocol:</span>
                <span className="uppercase">{ep.protocol}</span>
              </div>
              <div className="flex gap-4">
                <span className="text-neutral-400">Address:</span>
                <span>{ep.host}:{ep.port}</span>
              </div>
              <div className="flex gap-4">
                <span className="text-neutral-400">Pixels:</span>
                <span>{ep.start} – {ep.end}</span>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Audio */}
      <div>
        <h3 className="text-sm font-medium mb-2">Audio</h3>
        <div className="rounded bg-surface-2 border border-border p-3 text-xs space-y-1">
          <div className="flex gap-4">
            <span className="text-neutral-400">Sample Rate:</span>
            <span>{config.audio.sampleRate} Hz</span>
          </div>
          <div className="flex gap-4">
            <span className="text-neutral-400">Buffer Size:</span>
            <span>{config.audio.bufferSize}</span>
          </div>
          <div className="flex gap-4">
            <span className="text-neutral-400">Channels:</span>
            <span>{config.audio.channels}</span>
          </div>
        </div>
      </div>

      {/* Mapping */}
      <div>
        <label className="block text-xs text-neutral-400 mb-1">Pixel Mapping</label>
        <span className="text-sm font-mono">{config.mapping}</span>
      </div>
    </div>
  );
}
