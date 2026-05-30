use url::form_urlencoded;

/// Convert raw bytes into percent-encoded string (RFC-compliant)
fn encode_bytes(bytes: &[u8]) -> String {
    form_urlencoded::byte_serialize(bytes).collect()
}

#[derive(Debug)]
pub struct AnnounceRequest {
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub compact: bool,
}

pub fn build_announce_url(tracker_url: &str, req: &AnnounceRequest) -> String {
    let mut url = String::new();

    url.push_str(tracker_url);
    url.push('?');

    let info_hash = encode_bytes(&req.info_hash);
    let peer_id = encode_bytes(&req.peer_id);

    url.push_str(&format!("info_hash={}", info_hash));
    url.push('&');
    url.push_str(&format!("peer_id={}", peer_id));
    url.push('&');
    url.push_str(&format!("port={}", req.port));
    url.push('&');
    url.push_str(&format!("uploaded={}", req.uploaded));
    url.push('&');
    url.push_str(&format!("downloaded={}", req.downloaded));
    url.push('&');
    url.push_str(&format!("left={}", req.left));
    url.push('&');
    url.push_str(&format!("compact={}", if req.compact { 1 } else { 0 }));

    url
}
