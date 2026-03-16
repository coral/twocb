import { useEffect, useState } from "react";
import { usePatterns } from "../../stores/patterns";
import clsx from "clsx";

export default function PatternBrowser() {
  const { files, selected, dirty, fetchFiles, selectFile, createFile, deleteFile } = usePatterns();
  const [newName, setNewName] = useState("");
  const [showCreate, setShowCreate] = useState(false);

  useEffect(() => {
    fetchFiles();
  }, []);

  const handleCreate = async () => {
    const name = newName.endsWith(".js") ? newName : `${newName}.js`;
    await createFile(name);
    setNewName("");
    setShowCreate(false);
    selectFile(name);
  };

  return (
    <div className="p-3">
      <div className="flex items-center justify-between mb-2">
        <h2 className="text-xs font-medium text-neutral-400 uppercase tracking-wider">Patterns</h2>
        <button
          className="text-xs text-accent hover:text-accent-hover"
          onClick={() => setShowCreate(!showCreate)}
        >
          + New
        </button>
      </div>

      {showCreate && (
        <div className="flex gap-1 mb-2">
          <input
            className="flex-1 rounded bg-surface-2 border border-border px-2 py-1 text-xs outline-none focus:border-accent"
            placeholder="pattern.js"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleCreate()}
            autoFocus
          />
          <button
            className="text-xs bg-accent rounded px-2 py-1 text-white"
            onClick={handleCreate}
          >
            Create
          </button>
        </div>
      )}

      <div className="space-y-0.5">
        {files.map((f) => (
          <div
            key={f}
            className={clsx(
              "flex items-center gap-2 px-2 py-1.5 rounded text-xs cursor-pointer group",
              selected === f
                ? "bg-accent/20 text-accent"
                : "text-neutral-300 hover:bg-surface-2",
            )}
            onClick={() => selectFile(f)}
          >
            <span className="flex-1 font-mono truncate">
              {f}
              {selected === f && dirty && <span className="text-yellow-400 ml-1">*</span>}
            </span>
            <button
              className="text-neutral-600 hover:text-red-400 opacity-0 group-hover:opacity-100"
              onClick={(e) => {
                e.stopPropagation();
                if (confirm(`Delete ${f}?`)) deleteFile(f);
              }}
            >
              ×
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
