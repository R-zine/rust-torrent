use std::{
    fs::{self, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::Path,
};

pub fn prepare_download_file(path: &str, length: u64) -> std::io::Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;

    file.set_len(length)?;

    Ok(())
}

pub fn write_piece(
    path: &str,
    piece_index: usize,
    piece_length: usize,
    data: &[u8],
) -> std::io::Result<()> {
    let offset = (piece_index * piece_length) as u64;

    let mut file = OpenOptions::new().write(true).open(path)?;

    file.seek(SeekFrom::Start(offset))?;

    file.write_all(data)?;

    Ok(())
}
