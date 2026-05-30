use crate::download::piece::PieceState;

#[derive(Debug)]
pub struct DownloadManager {
    pieces: Vec<PieceState>,
}

impl DownloadManager {
    pub fn new(piece_count: usize) -> Self {
        Self {
            pieces: vec![PieceState::Missing; piece_count],
        }
    }
}

impl DownloadManager {
    pub fn piece_count(&self) -> usize {
        self.pieces.len()
    }

    pub fn completed_count(&self) -> usize {
        self.pieces
            .iter()
            .filter(|p| matches!(p, PieceState::Complete))
            .count()
    }

    pub fn missing_count(&self) -> usize {
        self.pieces
            .iter()
            .filter(|p| matches!(p, PieceState::Missing))
            .count()
    }

    pub fn mark_downloading(&mut self, piece: usize) {
        self.pieces[piece] = PieceState::Downloading;
    }

    pub fn mark_complete(&mut self, piece: usize) {
        self.pieces[piece] = PieceState::Complete;
    }

        pub fn next_missing_piece(&self) -> Option<usize> {
        self.pieces
            .iter()
            .position(|p| matches!(p, PieceState::Missing))
    }

pub fn downloading_count(&self) -> usize {
    self.pieces
        .iter()
        .filter(|p| matches!(
            p,
            super::piece::PieceState::Downloading
        ))
        .count()
}

pub fn completion_percent(&self) -> f64 {
    if self.pieces.is_empty() {
        return 0.0;
    }

    (self.completed_count() as f64
        / self.piece_count() as f64)
        * 100.0
}
}