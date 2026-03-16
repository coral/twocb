use anyhow::Result;
use log::{debug, error, info, warn};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

/// Packed RGB u8 pixel frames broadcast to all WebRTC peers.
/// Format: [u32 LE frame_index][R,G,B,R,G,B,...] = 4 + (pixel_count * 3) bytes
#[derive(Clone)]
pub struct PixelBroadcast {
    pub pixel_tx: broadcast::Sender<Arc<Vec<u8>>>,
}

impl PixelBroadcast {
    pub fn new() -> Self {
        let (pixel_tx, _) = broadcast::channel(4);
        Self { pixel_tx }
    }

    /// Convert RGBA f64 pixels to packed RGB u8 with frame index header
    pub fn pack_frame(frame_index: u32, pixels: &[vecmath::Vector4<f64>]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + pixels.len() * 3);
        buf.extend_from_slice(&frame_index.to_le_bytes());
        for px in pixels {
            buf.push((px[0].clamp(0.0, 1.0) * 255.0) as u8);
            buf.push((px[1].clamp(0.0, 1.0) * 255.0) as u8);
            buf.push((px[2].clamp(0.0, 1.0) * 255.0) as u8);
        }
        buf
    }
}

/// Manages active WebRTC peer connections for pixel streaming
pub struct PeerManager {
    peers: Vec<PeerEntry>,
}

struct PeerEntry {
    peer: Arc<RTCPeerConnection>,
    dc: Arc<webrtc::data_channel::RTCDataChannel>,
}

impl PeerManager {
    pub fn new() -> Self {
        Self { peers: Vec::new() }
    }

    /// Create a new peer connection from an SDP offer, return answer SDP
    pub async fn create_peer(
        &mut self,
        offer: RTCSessionDescription,
        mut pixel_rx: broadcast::Receiver<Arc<Vec<u8>>>,
    ) -> Result<(RTCSessionDescription, Arc<RTCPeerConnection>)> {
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()?;

        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine)?;

        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();

        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let peer = Arc::new(api.new_peer_connection(config).await?);

        // Create server-initiated data channel with unreliable/unordered (UDP semantics)
        let dc_init = RTCDataChannelInit {
            ordered: Some(false),
            max_retransmits: Some(0),
            ..Default::default()
        };
        let dc = peer
            .create_data_channel("pixels", Some(dc_init))
            .await?;

        // Spawn pixel forwarding task when data channel opens
        let dc_clone = dc.clone();
        dc.on_open(Box::new(move || {
            let dc = dc_clone.clone();
            Box::pin(async move {
                info!("WebRTC DataChannel 'pixels' opened");
                let dc_send = dc.clone();
                tokio::spawn(async move {
                    loop {
                        match pixel_rx.recv().await {
                            Ok(frame) => {
                                if let Err(e) = dc_send.send(&bytes::Bytes::copy_from_slice(&frame)).await {
                                    debug!("DataChannel send error: {}", e);
                                    break;
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                debug!("WebRTC pixel receiver lagged {} frames", n);
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                break;
                            }
                        }
                    }
                });
            })
        }));

        dc.on_close(Box::new(|| {
            Box::pin(async move {
                info!("WebRTC DataChannel closed");
            })
        }));

        // Set remote description (the browser's offer)
        peer.set_remote_description(offer).await?;

        // Create answer
        let answer = peer.create_answer(None).await?;
        peer.set_local_description(answer.clone()).await?;

        // Monitor connection state
        let peer_clone = peer.clone();
        peer.on_ice_connection_state_change(Box::new(move |state| {
            info!("ICE connection state: {:?}", state);
            Box::pin(async move {})
        }));

        self.peers.push(PeerEntry {
            peer: peer.clone(),
            dc,
        });

        Ok((answer, peer))
    }

    /// Add an ICE candidate to a specific peer
    pub async fn add_ice_candidate(
        peer: &RTCPeerConnection,
        candidate: RTCIceCandidateInit,
    ) -> Result<()> {
        peer.add_ice_candidate(candidate).await?;
        Ok(())
    }

    /// Remove disconnected peers
    pub async fn cleanup(&mut self) {
        self.peers.retain(|entry| {
            let state = entry.peer.ice_connection_state();
            !matches!(
                state,
                webrtc::ice_transport::ice_connection_state::RTCIceConnectionState::Disconnected
                    | webrtc::ice_transport::ice_connection_state::RTCIceConnectionState::Failed
                    | webrtc::ice_transport::ice_connection_state::RTCIceConnectionState::Closed
            )
        });
    }
}
