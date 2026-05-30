#[derive(Clone, Copy)]
pub struct Handshake {
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn build(&self) -> [u8; 68] {
        let mut buf = [0u8; 68];

        buf[0] = 19;
        buf[1..20].copy_from_slice(b"BitTorrent protocol");

        // reserved bytes already 0

        buf[28..48].copy_from_slice(&self.info_hash);
        buf[48..68].copy_from_slice(&self.peer_id);

        buf
    }
}