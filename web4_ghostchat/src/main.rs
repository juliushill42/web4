mod identity;
mod storage;
mod app;
mod transport;

use libp2p::{futures::StreamExt, gossipsub, swarm::SwarmEvent};
use std::error::Error;
use tokio::sync::mpsc;
use sha2::{Sha256, Digest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (ui_input_tx, mut ui_input_rx) = mpsc::channel::<String>(32);
    let (ui_output_tx, ui_output_rx) = mpsc::channel::<String>(32);
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    
    // NOTE: Hardcoded symmetric key for PoC. Must be exchanged out-of-band in production.
    let room_crypto = app::crypto::RoomCrypto::new("operation_ghost_key");
    let brain = storage::LocalBrain::init("web4_node.db")?;

    // Graceful Shutdown Channel handling
    tokio::spawn(async move {
        if let Err(_) = app::ui::run_ui(ui_input_tx, ui_output_rx).await {
            let _ = shutdown_tx.send(()).await;
        } else {
            let _ = shutdown_tx.send(()).await;
        }
    });
    
    #[cfg(feature = "ble")]
    {
        let ble_tx = ui_output_tx.clone();
        tokio::spawn(async move { let _ = transport::ble::run_ble_scanner(ble_tx).await; });
    }

    let mut swarm = transport::p2p::build_swarm().await?;
    let topic = gossipsub::IdentTopic::new("ghostchat-global");
    
    let _ = ui_output_tx.send("SYSTEM: Swarm Initialized. Awaiting peers...".to_string()).await;

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                break; // Exit cleanly, flushing DB locks
            }
            Some(msg) = ui_input_rx.recv() => {
                if let Ok(encrypted) = room_crypto.encrypt(&msg) {
                    let _ = brain.append_to_dag(&encrypted);
                    let _ = swarm.behaviour_mut().gossipsub.publish(topic.clone(), encrypted);
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(transport::p2p::GhostEvent::Mdns(libp2p::mdns::Event::Discovered(list))) => {
                    for (peer_id, _) in list {
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        let _ = ui_output_tx.send(format!("MESH: Found Peer {}", peer_id)).await;
                    }
                }
                SwarmEvent::Behaviour(transport::p2p::GhostEvent::Gossipsub(gossipsub::Event::Message { propagation_source: peer_id, message, .. })) => {
                    let _ = ui_output_tx.send(format!("[NETWORK TAP] Intercepted: {}...", &hex::encode(&message.data)[0..24])).await;
                    if let Ok(decrypted) = room_crypto.decrypt(&message.data) {
                        let mut hasher = Sha256::new();
                        hasher.update(peer_id.to_bytes());
                        let did = format!("did:web4:{}", hex::encode(hasher.finalize()));
                        
                        let _ = brain.append_to_dag(&message.data);
                        let _ = ui_output_tx.send(format!("[{}] says: {}", &did[..16], decrypted)).await;
                    }
                }
                _ => {}
            }
        }
    }
    
    Ok(())
}
