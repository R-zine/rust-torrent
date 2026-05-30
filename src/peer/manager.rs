use std::collections::HashMap;

use super::bitfield::Bitfield;

#[derive(Debug)]
pub struct PeerState {
    pub ip: String,
    pub port: u16,

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

        pub fn add_peer(
        &mut self,
        ip: String,
        port: u16,
    ) {
        let key = format!("{}:{}", ip, port);

        self.peers.insert(
            key,
            PeerState {
                ip,
                port,
                bitfield: None,
                choked: true,
            },
        );
    }

        pub fn update_bitfield(
        &mut self,
        ip: &str,
        port: u16,
        bitfield: Bitfield,
    ) {
        let key = format!("{}:{}", ip, port);

        if let Some(peer) =
            self.peers.get_mut(&key)
        {
            peer.bitfield = Some(bitfield);
        }
    }

       pub fn set_choked(
        &mut self,
        ip: &str,
        port: u16,
        choked: bool,
    ) {
        let key = format!("{}:{}", ip, port);

        if let Some(peer) =
            self.peers.get_mut(&key)
        {
            peer.choked = choked;
        }
    }

pub fn peers_with_piece(
    &self,
    piece_index: usize,
) -> Vec<&PeerState> {
    self.peers
        .values()
        .filter(|peer| {
            peer.bitfield
                .as_ref()
                .is_some_and(|b| {
                    b.has_piece(piece_index)
                })
        })
        .collect()
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
    self.peers
        .values()
        .filter(|peer| !peer.choked)
        .count()
}

pub fn total_advertised_pieces(&self) -> usize {
    self.peers
        .values()
        .filter_map(|peer| peer.bitfield.as_ref())
        .map(|bitfield| bitfield.piece_count())
        .sum()
}

}