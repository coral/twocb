use actix_web::{HttpRequest, HttpResponse, web};
use actix_ws::Message;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

use crate::rtc::{PeerManager, PixelBroadcast};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SignalMessage {
    #[serde(rename = "offer")]
    Offer { sdp: String },
    #[serde(rename = "answer")]
    Answer { sdp: String },
    #[serde(rename = "ice")]
    Ice { candidate: String, sdp_mid: Option<String>, sdp_mline_index: Option<u16> },
}

/// WebSocket handler for WebRTC signaling
pub async fn ws_signal(
    req: HttpRequest,
    stream: web::Payload,
    peer_manager: web::Data<Arc<Mutex<PeerManager>>>,
    pixel_broadcast: web::Data<PixelBroadcast>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let peer_manager = peer_manager.get_ref().clone();
    let pixel_broadcast = pixel_broadcast.get_ref().clone();

    // Use actix_web::rt::spawn because MessageStream is !Send
    actix_web::rt::spawn(async move {
        // Current peer connection for this WebSocket session
        let mut current_peer: Option<Arc<RTCPeerConnection>> = None;

        while let Some(Ok(msg)) = futures::StreamExt::next(&mut msg_stream).await {
            match msg {
                Message::Text(text) => {
                    let signal: SignalMessage = match serde_json::from_str(&text) {
                        Ok(s) => s,
                        Err(e) => {
                            error!("Invalid signal message: {}", e);
                            continue;
                        }
                    };

                    match signal {
                        SignalMessage::Offer { sdp } => {
                            info!("Received WebRTC offer");
                            let offer = RTCSessionDescription::offer(sdp).unwrap();
                            let pixel_rx = pixel_broadcast.pixel_tx.subscribe();

                            let mut pm = peer_manager.lock().await;
                            match pm.create_peer(offer, pixel_rx).await {
                                Ok((answer, peer)) => {
                                    // Set up ICE candidate gathering → send to browser
                                    let session_clone = session.clone();
                                    peer.on_ice_candidate(Box::new(move |candidate| {
                                        let mut session = session_clone.clone();
                                        Box::pin(async move {
                                            if let Some(c) = candidate {
                                                let json = c.to_json().unwrap();
                                                let msg = serde_json::json!({
                                                    "type": "ice",
                                                    "candidate": json.candidate,
                                                    "sdpMid": json.sdp_mid,
                                                    "sdpMLineIndex": json.sdp_mline_index,
                                                });
                                                let _ = session.text(msg.to_string()).await;
                                            }
                                        })
                                    }));

                                    current_peer = Some(peer);

                                    let answer_msg = serde_json::json!({
                                        "type": "answer",
                                        "sdp": answer.sdp,
                                    });
                                    let _ = session.text(answer_msg.to_string()).await;
                                    info!("Sent WebRTC answer");
                                }
                                Err(e) => {
                                    error!("Failed to create peer: {}", e);
                                }
                            }
                        }
                        SignalMessage::Ice { candidate, sdp_mid, sdp_mline_index } => {
                            if let Some(ref peer) = current_peer {
                                let init = RTCIceCandidateInit {
                                    candidate,
                                    sdp_mid: Some(sdp_mid.unwrap_or_default()),
                                    sdp_mline_index: Some(sdp_mline_index.unwrap_or(0)),
                                    username_fragment: Some(String::new()),
                                };
                                if let Err(e) = PeerManager::add_ice_candidate(peer, init).await {
                                    error!("Failed to add ICE candidate: {}", e);
                                }
                            }
                        }
                        SignalMessage::Answer { .. } => {
                            // Server doesn't expect answers
                        }
                    }
                }
                Message::Close(_) => {
                    info!("WebRTC signaling WebSocket closed");
                    break;
                }
                _ => {}
            }
        }

        // Cleanup: close peer connection
        if let Some(peer) = current_peer {
            let _ = peer.close().await;
        }
    });

    Ok(response)
}

/// WebSocket handler for state notifications (layer changes, param updates)
pub async fn ws_state(
    req: HttpRequest,
    stream: web::Payload,
    state_rx: web::Data<broadcast::Sender<String>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let mut rx = state_rx.subscribe();

    // Use actix_web::rt::spawn because MessageStream is !Send
    actix_web::rt::spawn(async move {
        loop {
            tokio::select! {
                // Forward server state changes to client
                Ok(msg) = rx.recv() => {
                    let _ = session.text(msg).await;
                }
                // Handle client messages
                msg = futures::StreamExt::next(&mut msg_stream) => {
                    match msg {
                        Some(Ok(Message::Text(_text))) => {
                            // Client-side state updates can be handled here
                        }
                        Some(Ok(Message::Close(_))) | None => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    Ok(response)
}
