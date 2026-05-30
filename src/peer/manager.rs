use std::collections::HashMap;

use super::bitfield::Bitfield;

#[derive(Debug)]
pub struct PeerState {
    pub bitfield: Option<Bitfield>,

    pub choked: bool,
}

pub struct PeerManager {
    peers: HashMap<String, PeerState>,
}

impl PeerManager {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    pub fn add_peer(&mut self, ip: String, port: u16) {
        let key = format!("{}:{}", ip, port);

        self.peers.insert(
            key,
            PeerState {
                bitfield: None,
                choked: true,
            },
        );
    }

    pub fn update_bitfield(&mut self, ip: &str, port: u16, bitfield: Bitfield) {
        let key = format!("{}:{}", ip, port);

        if let Some(peer) = self.peers.get_mut(&key) {
            peer.bitfield = Some(bitfield);
        }
    }

    pub fn set_choked(&mut self, ip: &str, port: u16, choked: bool) {
        let key = format!("{}:{}", ip, port);

        if let Some(peer) = self.peers.get_mut(&key) {
            peer.choked = choked;
        }
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn peers_with_bitfields(&self) -> usize {
        self.peers
            .values()
            .filter(|peer| peer.bitfield.is_some())
            .count()
    }

    pub fn unchoked_peers(&self) -> usize {
        self.peers.values().filter(|peer| !peer.choked).count()
    }

    pub fn peer_has_piece(&self, ip: &str, port: u16, piece_index: usize) -> bool {
        let key = format!("{}:{}", ip, port);

        self.peers
            .get(&key)
            .and_then(|peer| peer.bitfield.as_ref())
            .is_some_and(|bitfield| bitfield.has_piece(piece_index))
    }
}
