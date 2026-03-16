import { useConnection } from "../stores/connection";

type MessageHandler = (data: unknown) => void;

let socket: WebSocket | null = null;
let handlers: MessageHandler[] = [];

export function connectStateWs() {
  const wsUrl = useConnection.getState().wsUrl();
  if (socket && socket.readyState <= WebSocket.OPEN) return;

  socket = new WebSocket(`${wsUrl}/ws/state`);

  socket.onmessage = (ev) => {
    try {
      const data = JSON.parse(ev.data);
      handlers.forEach((h) => h(data));
    } catch {}
  };

  socket.onclose = () => {
    setTimeout(connectStateWs, 2000);
  };

  socket.onerror = () => {
    socket?.close();
  };
}

export function onStateMessage(handler: MessageHandler) {
  handlers.push(handler);
  return () => {
    handlers = handlers.filter((h) => h !== handler);
  };
}

export function disconnectStateWs() {
  socket?.close();
  socket = null;
  handlers = [];
}
