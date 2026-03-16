import { useEffect, useState } from "react";
import { useLayers } from "../../stores/layers";
import { useParams } from "../../stores/params";
import type { Link } from "../../types/api";
import clsx from "clsx";

export default function LayerStack() {
  const { links, fetchLayers } = useLayers();
  const fetchParams = useParams((s) => s.fetchParams);
  const selectedPattern = useParams((s) => s.patternName);
  const [expanded, setExpanded] = useState<string | null>(null);

  useEffect(() => {
    fetchLayers();
  }, []);

  return (
    <div className="p-3">
      <div className="flex items-center justify-between mb-2">
        <h2 className="text-xs font-medium text-neutral-400 uppercase tracking-wider">Layers</h2>
      </div>

      <div className="space-y-1">
        {links.map((link) => (
          <LayerCard
            key={link.name}
            link={link}
            expanded={expanded === link.name}
            onToggle={() => setExpanded(expanded === link.name ? null : link.name)}
            onSelectPattern={(name) => fetchParams(name)}
            selectedPattern={selectedPattern}
          />
        ))}
      </div>

      {links.length === 0 && (
        <p className="text-xs text-neutral-600 mt-2">No layers</p>
      )}
    </div>
  );
}

function LayerCard({
  link,
  expanded,
  onToggle,
  onSelectPattern,
  selectedPattern,
}: {
  link: Link;
  expanded: boolean;
  onToggle: () => void;
  onSelectPattern: (name: string) => void;
  selectedPattern: string | null;
}) {
  const setOpacity = useLayers((s) => s.setOpacity);
  const removeLayer = useLayers((s) => s.removeLayer);

  return (
    <div className="rounded bg-surface-2 border border-border">
      <div
        className="flex items-center gap-2 px-3 py-2 cursor-pointer select-none"
        onClick={onToggle}
      >
        <span className="text-xs text-neutral-500">{expanded ? "▾" : "▸"}</span>
        <span className="text-sm flex-1 truncate">{link.name}</span>
        <span className="text-xs text-neutral-500">{Math.round(link.opacity * 100)}%</span>
      </div>

      {expanded && (
        <div className="px-3 pb-3 space-y-2">
          {/* Opacity slider */}
          <div className="flex items-center gap-2">
            <label className="text-xs text-neutral-500 w-12">Opacity</label>
            <input
              type="range"
              min={0}
              max={1}
              step={0.01}
              value={link.opacity}
              onChange={(e) => setOpacity(link.name, Number(e.target.value))}
              className="flex-1 accent-accent h-1"
            />
          </div>

          {/* Steps */}
          <div className="space-y-1">
            {link.steps.map((step) => (
              <div
                key={step.pattern}
                className={clsx(
                  "flex items-center gap-2 px-2 py-1.5 rounded text-xs cursor-pointer",
                  selectedPattern === step.pattern
                    ? "bg-accent/20 text-accent"
                    : "bg-surface-3 text-neutral-300 hover:text-white",
                )}
                onClick={() => onSelectPattern(step.pattern)}
              >
                <span className="flex-1 truncate font-mono">{step.pattern}</span>
                <span className="text-neutral-600">{step.blendmode}</span>
              </div>
            ))}
          </div>

          <button
            className="text-xs text-red-400 hover:text-red-300"
            onClick={() => removeLayer(link.name)}
          >
            Remove layer
          </button>
        </div>
      )}
    </div>
  );
}
