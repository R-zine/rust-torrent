use super::announce::AnnounceRequest;
use super::announce::build_announce_url;
use crate::torrent::bencode::{Bencode, parse};
use reqwest;

#[derive(Debug)]
pub struct Peer {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug)]
pub struct TrackerResponse {
    pub peers: Vec<Peer>,
}

pub async fn announce(tracker_url: &str, req: &AnnounceRequest) -> Result<TrackerResponse, String> {
    let url = build_announce_url(tracker_url, req);

    let body = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    let parsed = parse(&body).map_err(|e| e.to_string())?;

    let dict = parsed
        .value
        .as_dictionary()
        .ok_or("tracker response not dict")?;

    let peers_raw = dict.get("peers").ok_or("missing peers")?;

    let peers_bytes = match &peers_raw.value {
        Bencode::String(b) => b,
        _ => return Err("invalid peers format".into()),
    };

    let mut peers = Vec::new();

    for chunk in peers_bytes.chunks(6) {
        if chunk.len() != 6 {
            continue;
        }

        let ip = format!("{}.{}.{}.{}", chunk[0], chunk[1], chunk[2], chunk[3]);

        let port = u16::from_be_bytes([chunk[4], chunk[5]]);

        peers.push(Peer { ip, port });
    }

    Ok(TrackerResponse {
        peers,
    })
}
