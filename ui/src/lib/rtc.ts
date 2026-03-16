import { useConnection } from "../stores/connection";
import { usePixels } from "../stores/pixels";
import type { SignalMessage } from "../types/ws";

let pc: RTCPeerConnection | null = null;
let signalWs: WebSocket | null = null;

export function connectPixelStream() {
  const wsUrl = useConnection.getState().wsUrl();

  pc = new RTCPeerConnection({
    iceServers: [{ urls: "stun:stun.l.google.com:19302" }],
  });

  // Server creates the DataChannel — browser receives it
  pc.ondatachannel = (event) => {
    const channel = event.channel;
    channel.binaryType = "arraybuffer";
    channel.onmessage = (msg) => handlePixelFrame(msg.data as ArrayBuffer);
    channel.onclose = () => console.log("Pixel DataChannel closed");
  };

  // Open signaling WebSocket
  signalWs = new WebSocket(`${wsUrl}/ws/signal`);

  signalWs.onopen = async () => {
    if (!pc) return;
    const offer = await pc.createOffer();
    await pc.setLocalDescription(offer);
    sendSignal({ type: "offer", sdp: offer.sdp! });
  };

  signalWs.onmessage = async (ev) => {
    const msg = JSON.parse(ev.data) as SignalMessage;

    if (msg.type === "answer") {
      await pc?.setRemoteDescription(new RTCSessionDescription({ type: "answer", sdp: msg.sdp }));
    } else if (msg.type === "ice") {
      await pc?.addIceCandidate(
        new RTCIceCandidate({
          candidate: msg.candidate,
          sdpMid: msg.sdpMid,
          sdpMLineIndex: msg.sdpMLineIndex,
        }),
      );
    }
  };

  // Send ICE candidates to server
  pc.onicecandidate = (ev) => {
    if (ev.candidate) {
      sendSignal({
        type: "ice",
        candidate: ev.candidate.candidate,
        sdpMid: ev.candidate.sdpMid ?? undefined,
        sdpMLineIndex: ev.candidate.sdpMLineIndex ?? undefined,
      });
    }
  };

  pc.oniceconnectionstatechange = () => {
    if (pc?.iceConnectionState === "disconnected" || pc?.iceConnectionState === "failed") {
      disconnectPixelStream();
      // Reconnect after brief delay
      setTimeout(connectPixelStream, 2000);
    }
  };
}

function sendSignal(msg: SignalMessage) {
  if (signalWs?.readyState === WebSocket.OPEN) {
    signalWs.send(JSON.stringify(msg));
  }
}

function handlePixelFrame(buffer: ArrayBuffer) {
  // Skip first 4 bytes (u32 LE frame index)
  const rgbData = new Uint8Array(buffer, 4);
  usePixels.getState().updateColors(rgbData);
}

export function disconnectPixelStream() {
  pc?.close();
  pc = null;
  signalWs?.close();
  signalWs = null;
}
