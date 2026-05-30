#[derive(Debug)]
pub struct TorrentMetadata {
    pub announce: String,
    pub info_bytes: Vec<u8>,

    pub piece_length: u64,
    pub piece_hashes: Vec<[u8; 20]>,
    pub total_length: u64,

    pub name: String,
}
