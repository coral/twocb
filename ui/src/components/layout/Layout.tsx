import { useState } from "react";
import { clsx } from "clsx";
import ConnectionStatusDot from "../connect/ConnectionStatus";
import LayerStack from "../layers/LayerStack";
import ParamPanel from "../params/ParamPanel";
import PatternBrowser from "../editor/PatternBrowser";
import PixelPreview from "../preview/PixelPreview";
import PatternEditor from "../editor/PatternEditor";
import SettingsPanel from "../settings/SettingsPanel";

type Tab = "preview" | "editor" | "settings";

export default function Layout() {
  const [tab, setTab] = useState<Tab>("preview");

  return (
    <div className="flex h-screen flex-col">
      {/* Top bar */}
      <header className="flex items-center justify-between border-b border-border bg-surface-1 px-4 h-11 shrink-0">
        <div className="flex items-center gap-6">
          <span className="font-semibold text-sm tracking-wide">twocb</span>
          <nav className="flex gap-1">
            {(["preview", "editor", "settings"] as Tab[]).map((t) => (
              <button
                key={t}
                onClick={() => setTab(t)}
                className={clsx(
                  "px-3 py-1 text-xs rounded capitalize",
                  tab === t
                    ? "bg-surface-3 text-white"
                    : "text-neutral-500 hover:text-neutral-300",
                )}
              >
                {t}
              </button>
            ))}
          </nav>
        </div>
        <ConnectionStatusDot />
      </header>

      {/* Main area */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <aside className="w-72 border-r border-border bg-surface-1 overflow-y-auto shrink-0">
          {tab === "preview" && (
            <>
              <LayerStack />
              <ParamPanel />
            </>
          )}
          {tab === "editor" && <PatternBrowser />}
          {tab === "settings" && null}
        </aside>

        {/* Content */}
        <main className="flex-1 overflow-hidden">
          {tab === "preview" && <PixelPreview />}
          {tab === "editor" && <PatternEditor />}
          {tab === "settings" && <SettingsPanel />}
        </main>
      </div>
    </div>
  );
}
