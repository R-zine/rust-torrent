pub mod bencode;
pub mod hash;
pub mod metadata;

mod reader;

pub use reader::read_torrent;
