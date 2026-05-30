#[derive(Debug, Clone)]
pub struct Bitfield {
    bytes: Vec<u8>,
}

impl Bitfield {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn has_piece(&self, piece_index: usize) -> bool {
        let byte_index = piece_index / 8;

        if byte_index >= self.bytes.len() {
            return false;
        }

        let bit_index = 7 - (piece_index % 8);

        (self.bytes[byte_index] & (1 << bit_index)) != 0
    }

    pub fn piece_count(&self) -> usize {
        self.bytes
            .iter()
            .map(|byte| byte.count_ones() as usize)
            .sum()
    }
}