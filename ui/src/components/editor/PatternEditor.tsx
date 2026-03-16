import { useEffect, useCallback } from "react";
import Editor from "@monaco-editor/react";
import { usePatterns } from "../../stores/patterns";

// Type definitions for pattern API intellisense
const PATTERN_DEFS = `
declare var pixelCount: number;
declare var world: {
  framerate: number;
  delta: number;
  phase: number;
  bar: number;
  bpm: number;
  colorchord: {
    notes: Array<{ amp: number; amp_filt: number; position: number }>;
    folded: number[];
  };
};
declare var state: Record<string, any>;
declare function beforeRender(frame: any, delta: number): void;
declare function render3D(index: number, x: number, y: number, z: number): void;
declare function hsv(index: number, h: number, s: number, v: number): void;
declare function rgb(index: number, r: number, g: number, b: number): void;
declare function rgba(index: number, r: number, g: number, b: number, a: number): void;
declare function sin(phase: number, cycle?: number): number;
declare function cos(phase: number, cycle?: number): number;
declare function triangle(phase: number, cycle?: number): number;
declare function square(phase: number, cycle?: number): number;
declare var PI: number;
declare var PI2: number;
declare function random(): number;
declare function abs(x: number): number;
declare function max(...values: number[]): number;
declare function floor(x: number): number;
`;

export default function PatternEditor() {
  const { selected, content, dirty, setContent, save } = usePatterns();

  const handleSave = useCallback(
    (e?: KeyboardEvent) => {
      if (e) {
        if ((e.metaKey || e.ctrlKey) && e.key === "s") {
          e.preventDefault();
          save();
        }
      } else {
        save();
      }
    },
    [save],
  );

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault();
        save();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [save]);

  if (!selected) {
    return (
      <div className="flex items-center justify-center h-full text-neutral-600 text-sm">
        Select a pattern to edit
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border bg-surface-1">
        <span className="text-sm font-mono">
          {selected}
          {dirty && <span className="text-yellow-400 ml-2">unsaved</span>}
        </span>
        <button
          className="text-xs bg-accent hover:bg-accent-hover rounded px-3 py-1 text-white disabled:opacity-50"
          onClick={() => save()}
          disabled={!dirty}
        >
          Save
        </button>
      </div>

      {/* Editor */}
      <div className="flex-1">
        <Editor
          language="javascript"
          theme="vs-dark"
          value={content}
          onChange={(v) => setContent(v ?? "")}
          beforeMount={(monaco) => {
            monaco.languages.typescript.javascriptDefaults.addExtraLib(
              PATTERN_DEFS,
              "twocb-pattern.d.ts",
            );
          }}
          options={{
            fontSize: 13,
            fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
            minimap: { enabled: false },
            lineNumbers: "on",
            scrollBeyondLastLine: false,
            padding: { top: 12 },
            tabSize: 2,
          }}
        />
      </div>
    </div>
  );
}
