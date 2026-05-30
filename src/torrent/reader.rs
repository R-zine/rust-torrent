use std::fs;
use std::io;

use super::bencode::{Bencode, parse};
use crate::torrent::metadata::TorrentMetadata;

use std::fmt;

#[derive(Debug)]
pub enum TorrentError {
    Io(io::Error),
    Parse(String),
}

impl fmt::Display for TorrentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TorrentError::Io(err) => {
                write!(f, "IO error: {}", err)
            }

            TorrentError::Parse(msg) => {
                write!(f, "Parse error: {}", msg)
            }
        }
    }
}

impl From<io::Error> for TorrentError {
    fn from(err: io::Error) -> Self {
        TorrentError::Io(err)
    }
}

impl From<String> for TorrentError {
    fn from(err: String) -> Self {
        TorrentError::Parse(err)
    }
}

pub fn read_torrent(path: &str) -> Result<TorrentMetadata, TorrentError> {
    let bytes = fs::read(path)?;

    let root = parse(&bytes)?;

    let dict = match root.value {
        Bencode::Dictionary(ref d) => d,
        _ => {
            return Err(TorrentError::Parse(
                "torrent root must be a dictionary".into(),
            ));
        }
    };

    let announce = dict
        .get("announce")
        .ok_or_else(|| TorrentError::Parse("missing announce".into()))?;

    let announce = match &announce.value {
        Bencode::String(bytes) => String::from_utf8_lossy(bytes).to_string(),
        _ => return Err(TorrentError::Parse("announce must be a string".into())),
    };

    let info = dict
        .get("info")
        .ok_or_else(|| TorrentError::Parse("missing info".into()))?;

    let info_dict = match &info.value {
        Bencode::Dictionary(dict) => dict,
        _ => return Err(TorrentError::Parse("info must be a dictionary".into())),
    };

    let name = info_dict
        .get("name")
        .ok_or_else(|| TorrentError::Parse("missing name".into()))?;

    let name = match &name.value {
        Bencode::String(bytes) => String::from_utf8_lossy(bytes).to_string(),
        _ => return Err(TorrentError::Parse("name must be a string".into())),
    };

    let piece_length = info_dict
        .get("piece length")
        .ok_or_else(|| TorrentError::Parse("missing piece length".into()))?;

    let piece_length = match &piece_length.value {
        Bencode::Integer(i) => *i as u64,
        _ => {
            return Err(TorrentError::Parse(
                "piece length must be an integer".into(),
            ));
        }
    };

    let total_length = info_dict
        .get("length")
        .ok_or_else(|| TorrentError::Parse("missing length".into()))?;

    let total_length = match &total_length.value {
        Bencode::Integer(i) => *i as u64,
        _ => return Err(TorrentError::Parse("length must be an integer".into())),
    };

    let pieces = info_dict
        .get("pieces")
        .ok_or_else(|| TorrentError::Parse("missing pieces".into()))?;

    let pieces_bytes = match &pieces.value {
        Bencode::String(bytes) => bytes,
        _ => return Err(TorrentError::Parse("pieces must be a string".into())),
    };

    if pieces_bytes.len() % 20 != 0 {
        return Err(TorrentError::Parse("pieces field is malformed".into()));
    }

    let mut piece_hashes = Vec::new();

    for chunk in pieces_bytes.chunks_exact(20) {
        let mut hash = [0u8; 20];
        hash.copy_from_slice(chunk);
        piece_hashes.push(hash);
    }

    let info_bytes = bytes[info.start..info.end].to_vec();

    Ok(TorrentMetadata {
        announce,
        info_bytes,

        piece_length,
        piece_hashes,
        total_length,

        name,
    })
}
