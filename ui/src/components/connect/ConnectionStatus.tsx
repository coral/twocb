import { useConnection, type ConnectionStatus as Status } from "../../stores/connection";
import clsx from "clsx";

const colors: Record<Status, string> = {
  disconnected: "bg-neutral-500",
  connecting: "bg-yellow-400 animate-pulse",
  connected: "bg-green-400",
  error: "bg-red-400",
};

export default function ConnectionStatusDot() {
  const status = useConnection((s) => s.status);
  const host = useConnection((s) => s.host);
  const port = useConnection((s) => s.port);

  return (
    <div className="flex items-center gap-2 text-xs text-neutral-400">
      <div className={clsx("w-2 h-2 rounded-full", colors[status])} />
      <span>
        {status === "connected" ? `${host}:${port}` : status}
      </span>
    </div>
  );
}
