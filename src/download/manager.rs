const REQUEST_TIMEOUT_SECS: u64 = 10;
const RETRY_INTERVAL_SECS: u64 = 2;

use std::sync::Arc;
use std::{collections::HashMap, time::Instant};
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};

use crate::storage;
use crate::{
    download::{
        piece::PieceState,
        piece_buffer::{BLOCK_SIZE, PieceBuffer},
    },
    peer::manager::PeerManager,
};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct BlockRequestKey {
    pub piece_index: usize,
    pub begin: u32,
}

#[derive(Debug)]
pub struct DownloadManager {
    pub output_file: String,
    pub final_file_name: String,

    pub pieces: Vec<PieceState>,

    pub piece_length: usize,
    pub piece_hashes: Vec<[u8; 20]>,
    pub in_flight: HashMap<BlockRequestKey, Instant>,

    pub finalized: bool,
}

#[derive(Debug, Clone)]
pub struct BlockAssignment {
    pub piece_index: usize,
    pub begin: u32,
    pub length: u32,
}

impl DownloadManager {
    pub fn new(
        output_file: String,
        final_file_name: String,
        piece_count: usize,
        piece_length: usize,
        piece_hashes: Vec<[u8; 20]>,
    ) -> Self {
        Self {
            output_file,
            final_file_name,
            pieces: vec![PieceState::Missing; piece_count],
            piece_length,
            piece_hashes,
            in_flight: HashMap::new(),
            finalized: false,
        }
    }

    pub fn piece_count(&self) -> usize {
        self.pieces.len()
    }

    pub fn completed_count(&self) -> usize {
        self.pieces
            .iter()
            .filter(|p| matches!(p, PieceState::Done))
            .count()
    }

    pub fn missing_count(&self) -> usize {
        self.pieces
            .iter()
            .filter(|p| matches!(p, PieceState::Missing))
            .count()
    }

    pub fn downloading_count(&self) -> usize {
        self.pieces
            .iter()
            .filter(|p| matches!(p, PieceState::Downloading(_)))
            .count()
    }

    pub fn completion_percent(&self) -> f64 {
        if self.pieces.is_empty() {
            return 0.0;
        }

        (self.completed_count() as f64 / self.piece_count() as f64) * 100.0
    }

    pub fn insert_block(
        &mut self,
        index: usize,
        begin: u32,
        data: Vec<u8>,
    ) -> Result<Option<bool>, std::io::Error> {
        match &mut self.pieces[index] {
            PieceState::Downloading(buffer) => {
                buffer.insert_block(begin, data);
                if buffer.verify() {
                    let bytes = buffer.assemble();

                    storage::writer::write_piece(
                        &self.output_file,
                        index,
                        self.piece_length,
                        &bytes,
                    )?;

                    self.pieces[index] = PieceState::Done;

                    return Ok(Some(true));
                }

                if buffer.is_complete() {
                    let valid = buffer.verify();

                    if valid {
                        self.pieces[index] = PieceState::Done;

                        return Ok(Some(true));
                    } else {
                        self.pieces[index] = PieceState::Missing;
                        return Ok(Some(false));
                    }
                }

                Ok(None)
            }

            _ => Ok(None),
        }
    }

    pub fn assign_block_for_peer(
        &mut self,
        peer_manager: &PeerManager,
        ip: &str,
        port: u16,
    ) -> Option<BlockAssignment> {
        for piece_index in 0..self.pieces.len() {
            let piece = &mut self.pieces[piece_index];

            let buffer = match piece {
                PieceState::Downloading(b) => b,
                PieceState::Missing => {
                    // create buffer lazily
                    if !peer_manager.peer_has_piece(ip, port, piece_index) {
                        continue;
                    }

                    let buffer =
                        PieceBuffer::new(self.piece_length, self.piece_hashes[piece_index]);

                    *piece = PieceState::Downloading(buffer);

                    match piece {
                        PieceState::Downloading(b) => b,
                        _ => unreachable!(),
                    }
                }
                PieceState::Done => continue,
            };

            if !peer_manager.peer_has_piece(ip, port, piece_index) {
                continue;
            }

            if let Some(begin) = buffer.next_missing_block() {
                let key = BlockRequestKey { piece_index, begin };

                if self.in_flight.contains_key(&key) {
                    continue;
                }

                let length = BLOCK_SIZE.min(self.piece_length - (begin as usize)) as u32;

                return Some(BlockAssignment {
                    piece_index,
                    begin,
                    length,
                });
            }
        }

        None
    }

    pub fn mark_requested(&mut self, piece: usize, begin: u32) {
        let key = BlockRequestKey {
            piece_index: piece,
            begin,
        };

        self.in_flight.insert(key, Instant::now());
    }

    pub fn mark_received(&mut self, piece: usize, begin: u32) {
        let key = BlockRequestKey {
            piece_index: piece,
            begin,
        };

        self.in_flight.remove(&key);
    }

    pub fn cleanup_timeouts(&mut self, timeout_secs: u64) {
        use std::time::Instant;

        let now = Instant::now();

        let timed_out: Vec<_> = self
            .in_flight
            .iter()
            .filter(|(_, t)| now.duration_since(**t).as_secs() > timeout_secs)
            .map(|(k, _)| k.clone())
            .collect();

        for key in timed_out {
            self.in_flight.remove(&key);

            let piece = &mut self.pieces[key.piece_index];

            // Only reset if still downloading
            if let PieceState::Downloading(buffer) = piece {
                if !buffer.is_complete() {
                    // allow retry of missing blocks
                    println!("retrying piece {} block {}", key.piece_index, key.begin);
                }
            }
        }
    }

    pub async fn start_retry_scheduler(manager: Arc<Mutex<DownloadManager>>) {
        loop {
            sleep(Duration::from_secs(RETRY_INTERVAL_SECS)).await;

            let mut dm = manager.lock().await;

            let before = dm.in_flight.len();

            dm.cleanup_timeouts(REQUEST_TIMEOUT_SECS);

            let after = dm.in_flight.len();

            if before != after {
                println!(
                    "retry scheduler: cleared {} timed-out requests",
                    before - after
                );
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        self.completed_count() == self.piece_count()
    }
}
