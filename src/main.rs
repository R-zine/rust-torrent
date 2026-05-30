mod torrent;
mod tracker;
mod peer;
mod download;

use clap::Parser;

use std::sync::Arc;
use tokio::sync::Mutex;

use tokio::time::{sleep, Duration};

#[derive(Parser)]
struct Args {
    torrent_file: String,
}

#[tokio::main]
async fn main() {
    let metadata = torrent::read_torrent(&Args::parse().torrent_file)
        .expect("failed to read torrent");

    let peer_manager =
    Arc::new(Mutex::new(
        peer::manager::PeerManager::new(),
    ));

let download_manager =
    Arc::new(Mutex::new(
        download::manager::DownloadManager::new(
            metadata.piece_hashes.len(),
        ),
    ));

    let peer_id = tracker::peer_id::generate_peer_id();
    let info_hash = torrent::hash::info_hash(&metadata.info_bytes);

    let req = tracker::announce::AnnounceRequest {
        info_hash,
        peer_id,
        port: 6881,
        uploaded: 0,
        downloaded: 0,
        left: 0,
        compact: true,
    };

let response = tracker::client::announce(&metadata.announce, &req)
    .await
    .expect("tracker request failed");

    let peers: Vec<_> = response
        .peers
        .into_iter()
        .map(|p| peer::connection::PeerConnection {
            ip: p.ip,
            port: p.port,
            bitfield: None,
            choked: true,
        })
        .collect();

    let handshake = peer::handshake::Handshake {
        info_hash,
        peer_id,
    };

    let pm = Arc::clone(&peer_manager);
let dm = Arc::clone(&download_manager);

tokio::spawn(async move {
    loop {
        {
            let pm = pm.lock().await;
            let dm = dm.lock().await;

            println!(
                "\n=== STATUS ===\n\
                 peers: {}\n\
                 bitfields: {}\n\
                 unchoked: {}\n\
                 pieces: {}/{}\n\
                 downloading: {}\n\
                 missing: {}\n\
                 progress: {:.2}%\n",
                pm.peer_count(),
                pm.peers_with_bitfields(),
                pm.unchoked_peers(),
                dm.completed_count(),
                dm.piece_count(),
                dm.downloading_count(),
                dm.missing_count(),
                dm.completion_percent(),
            );
        }

        sleep(Duration::from_secs(5)).await;
    }
});

peer::connect_to_peers(
    peers,
    handshake,
    Arc::clone(&peer_manager),
    Arc::clone(&download_manager),
)
.await;
}