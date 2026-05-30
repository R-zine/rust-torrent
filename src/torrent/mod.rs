pub mod bencode;
pub mod metadata;
pub mod hash;

mod reader;

pub use reader::read_torrent;