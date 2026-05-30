#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceState {
    Missing,
    Downloading,
    Complete,
}