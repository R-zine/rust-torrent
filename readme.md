# Rust Torrent Client

A BitTorrent client from scratch in Rust.

## Project Structure

```text
src/
├── download/
├── peer/
├── storage/
├── torrent/
├── tracker/
└── main.rs
```

## Running

```bash
cargo run -- #torrent_path#
```

The client will:

1. Read the torrent file
2. Contact the tracker
3. Discover peers
4. Connect to peers
5. Download pieces
6. Verify piece hashes
7. Save the completed file

## Download Directories

```text
torrents/
├── downloading/
└── done/
```

Files are downloaded into `downloading` and moved to `done` when the torrent is complete.

## Disclaimer

This project is intended for educational purposes.

Only download content that you have the legal right to access and distribute.
