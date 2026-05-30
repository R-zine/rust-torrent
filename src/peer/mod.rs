pub mod bitfield;
pub mod connection;
pub mod handshake;
pub mod manager;
pub mod protocol;

use std::sync::Arc;

use tokio::{sync::Mutex, task};

use crate::download::manager::DownloadManager;

use self::{connection::PeerConnection, handshake::Handshake, manager::PeerManager};

pub type SharedPeerManager = Arc<Mutex<PeerManager>>;
pub type SharedDownloadManager = Arc<Mutex<DownloadManager>>;

pub async fn connect_to_peers(
    peers: Vec<PeerConnection>,
    handshake: Handshake,
    peer_manager: SharedPeerManager,
    download_manager: SharedDownloadManager,
) {
    let mut handles = Vec::new();

    for peer in peers {
        let hs = handshake;

        let pm = Arc::clone(&peer_manager);
        let dm = Arc::clone(&download_manager);

        let ip = peer.ip.clone();
        let port = peer.port;

        let handle = task::spawn(async move {
            match peer.connect_and_handshake(hs, pm, dm).await {
                Ok(_) => {
                    println!("connected to {}:{}", ip, port);
                }

                Err(e) => {
                    println!("failed {}:{} -> {}", ip, port, e);
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
}
