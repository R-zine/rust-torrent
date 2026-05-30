use rand::RngExt;

/// Generates a 20-byte BitTorrent peer_id
///
/// Format:
/// -TR0001- + 12 random bytes
pub fn generate_peer_id() -> [u8; 20] {
    let mut rng = rand::rng();

    let mut id = [0u8; 20];

    // Client prefix: "-TR0001-"
    let prefix = b"-TR0001-";
    id[..8].copy_from_slice(prefix);

    // Fill remaining bytes with random ASCII-ish chars
    for byte in &mut id[8..] {
        *byte = rng.random_range(b'a'..=b'z');
    }

    id
}
