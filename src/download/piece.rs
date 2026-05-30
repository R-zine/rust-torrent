use super::piece_buffer::PieceBuffer;

#[derive(Debug, Clone)]
pub enum PieceState {
    Missing,
    Downloading(PieceBuffer),
    Done,
}
