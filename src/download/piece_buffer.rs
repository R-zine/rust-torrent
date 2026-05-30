use sha1::{Digest, Sha1};

pub const BLOCK_SIZE: usize = 16 * 1024;

#[derive(Debug, Clone)]
pub struct PieceBuffer {
    pub length: usize,
    pub expected_hash: [u8; 20],

    pub blocks: Vec<Option<Vec<u8>>>,
    pub received_bytes: usize,
}

impl PieceBuffer {
    pub fn new(length: usize, expected_hash: [u8; 20]) -> Self {
        let block_count = (length + BLOCK_SIZE - 1) / BLOCK_SIZE;

        Self {
            length,
            expected_hash,
            blocks: vec![None; block_count],
            received_bytes: 0,
        }
    }

    pub fn insert_block(&mut self, begin: u32, data: Vec<u8>) {
        let index = (begin as usize) / BLOCK_SIZE;

        if index >= self.blocks.len() {
            return;
        }

        if self.blocks[index].is_some() {
            return;
        }

        self.received_bytes += data.len();
        self.blocks[index] = Some(data);
    }

    pub fn is_complete(&self) -> bool {
        self.blocks.iter().all(|b| b.is_some())
    }

    pub fn assemble(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.length);

        for block in &self.blocks {
            if let Some(data) = block {
                result.extend_from_slice(data);
            }
        }

        result.truncate(self.length);
        result
    }

    pub fn verify(&self) -> bool {
        let data = self.assemble();

        let mut hasher = Sha1::new();
        hasher.update(&data);

        let result = hasher.finalize();

        let mut actual = [0u8; 20];
        actual.copy_from_slice(&result);

        actual == self.expected_hash
    }

    pub fn next_missing_block(&self) -> Option<u32> {
        for (i, block) in self.blocks.iter().enumerate() {
            if block.is_none() {
                return Some((i * BLOCK_SIZE) as u32);
            }
        }
        None
    }
}
