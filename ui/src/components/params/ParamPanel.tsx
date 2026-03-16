import { useParams } from "../../stores/params";
import type { PatternParam, ParamValue } from "../../types/api";

export default function ParamPanel() {
  const { params, patternName } = useParams();

  if (!patternName || params.length === 0) return null;

  return (
    <div className="p-3 border-t border-border">
      <h2 className="text-xs font-medium text-neutral-400 uppercase tracking-wider mb-2">
        Parameters — {patternName}
      </h2>
      <div className="space-y-3">
        {params.map((p) => (
          <ParamControl key={p.name} param={p} />
        ))}
      </div>
    </div>
  );
}

function ParamControl({ param }: { param: PatternParam }) {
  const setParam = useParams((s) => s.setParam);

  switch (param.kind) {
    case "Slider":
      return (
        <SliderControl
          name={param.name}
          value={"Float" in param.value ? param.value.Float : 0.5}
          onChange={(v) => setParam(param.name, { Float: v })}
        />
      );
    case "Toggle":
      return (
        <ToggleControl
          name={param.name}
          value={"Bool" in param.value ? param.value.Bool : false}
          onChange={(v) => setParam(param.name, { Bool: v })}
        />
      );
    case "Number":
      return (
        <NumberControl
          name={param.name}
          value={"Float" in param.value ? param.value.Float : 0}
          onChange={(v) => setParam(param.name, { Float: v })}
        />
      );
    case "HsvPicker":
    case "RgbPicker": {
      const c = "Color3" in param.value ? param.value.Color3 : [0, 0, 0] as [number, number, number];
      return (
        <ColorControl
          name={param.name}
          kind={param.kind}
          value={c}
          onChange={(v) => setParam(param.name, { Color3: v })}
        />
      );
    }
  }
}

function SliderControl({ name, value, onChange }: { name: string; value: number; onChange: (v: number) => void }) {
  return (
    <div>
      <div className="flex justify-between text-xs text-neutral-400 mb-1">
        <span>{name}</span>
        <span>{value.toFixed(2)}</span>
      </div>
      <input
        type="range"
        min={0}
        max={1}
        step={0.01}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="w-full accent-accent h-1"
      />
    </div>
  );
}

function ToggleControl({ name, value, onChange }: { name: string; value: boolean; onChange: (v: boolean) => void }) {
  return (
    <label className="flex items-center gap-2 text-xs text-neutral-400 cursor-pointer">
      <input
        type="checkbox"
        checked={value}
        onChange={(e) => onChange(e.target.checked)}
        className="accent-accent"
      />
      {name}
    </label>
  );
}

function NumberControl({ name, value, onChange }: { name: string; value: number; onChange: (v: number) => void }) {
  return (
    <div>
      <label className="text-xs text-neutral-400 mb-1 block">{name}</label>
      <input
        type="number"
        step={0.01}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="w-full rounded bg-surface-3 border border-border px-2 py-1 text-xs outline-none focus:border-accent"
      />
    </div>
  );
}

function ColorControl({
  name,
  kind,
  value,
  onChange,
}: {
  name: string;
  kind: "HsvPicker" | "RgbPicker";
  value: [number, number, number];
  onChange: (v: [number, number, number]) => void;
}) {
  // Simple RGB color input — converts HSV ↔ RGB as needed
  const rgbHex =
    kind === "RgbPicker"
      ? `#${value.map((v) => Math.round(v * 255).toString(16).padStart(2, "0")).join("")}`
      : hsvToHex(value[0], value[1], value[2]);

  return (
    <div>
      <label className="text-xs text-neutral-400 mb-1 block">{name}</label>
      <input
        type="color"
        value={rgbHex}
        onChange={(e) => {
          const hex = e.target.value;
          const r = parseInt(hex.slice(1, 3), 16) / 255;
          const g = parseInt(hex.slice(3, 5), 16) / 255;
          const b = parseInt(hex.slice(5, 7), 16) / 255;
          if (kind === "RgbPicker") {
            onChange([r, g, b]);
          } else {
            onChange(rgbToHsv(r, g, b));
          }
        }}
        className="w-full h-8 rounded border border-border cursor-pointer"
      />
    </div>
  );
}

function hsvToHex(h: number, s: number, v: number): string {
  const c = v * s;
  const x = c * (1 - Math.abs(((h * 6) % 2) - 1));
  const m = v - c;
  let r = 0, g = 0, b = 0;
  const i = Math.floor(h * 6) % 6;
  if (i === 0) { r = c; g = x; }
  else if (i === 1) { r = x; g = c; }
  else if (i === 2) { g = c; b = x; }
  else if (i === 3) { g = x; b = c; }
  else if (i === 4) { r = x; b = c; }
  else { r = c; b = x; }
  return `#${[r + m, g + m, b + m].map((v) => Math.round(v * 255).toString(16).padStart(2, "0")).join("")}`;
}

function rgbToHsv(r: number, g: number, b: number): [number, number, number] {
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const d = max - min;
  let h = 0;
  const s = max === 0 ? 0 : d / max;
  const v = max;
  if (d !== 0) {
    if (max === r) h = ((g - b) / d + 6) % 6;
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
    h /= 6;
  }
  return [h, s, v];
}
