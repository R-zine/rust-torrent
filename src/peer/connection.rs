use std::io;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::peer::{
    SharedDownloadManager, SharedPeerManager,
    handshake::Handshake,
    protocol::{self, PeerMessage},
};

use super::bitfield::Bitfield;

pub struct PeerConnection {
    pub ip: String,
    pub port: u16,

    pub bitfield: Option<Bitfield>,
    pub choked: bool,
}

impl PeerConnection {
    async fn request_next_block(
        &self,
        stream: &mut TcpStream,
        peer_manager: &SharedPeerManager,
        download_manager: &SharedDownloadManager,
    ) -> io::Result<()> {
        let assignment = {
            let pm = peer_manager.lock().await;
            let mut dm = download_manager.lock().await;

            dm.assign_block_for_peer(&pm, &self.ip, self.port)
        };

        if let Some(block) = assignment {
            {
                let mut dm = download_manager.lock().await;

                dm.mark_requested(block.piece_index, block.begin);
            }

            stream
                .write_all(&protocol::request_message(
                    block.piece_index as u32,
                    block.begin,
                    block.length,
                ))
                .await?;
        }

        Ok(())
    }

    pub async fn connect_and_handshake(
        mut self,
        handshake: Handshake,
        peer_manager: SharedPeerManager,
        download_manager: SharedDownloadManager,
    ) -> io::Result<()> {
        let addr = format!("{}:{}", self.ip, self.port);

        println!("connecting {}", addr);

        let mut stream = TcpStream::connect(&addr).await?;

        peer_manager
            .lock()
            .await
            .add_peer(self.ip.clone(), self.port);

        stream.write_all(&handshake.build()).await?;

        let mut resp = [0u8; 68];
        stream.read_exact(&mut resp).await?;

        if resp[0] != 19 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid handshake",
            ));
        }

        if &resp[28..48] != handshake.info_hash.as_slice() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "info_hash mismatch",
            ));
        }

        println!("{}:{} handshake OK", self.ip, self.port);

        stream.write_all(&protocol::interested_message()).await?;

        println!("{}:{} interested sent", self.ip, self.port);

        loop {
            let msg = protocol::read_message(&mut stream)
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            match msg {
                PeerMessage::KeepAlive => {}

                PeerMessage::Bitfield(bytes) => {
                    let bitfield = Bitfield::new(bytes);

                    println!(
                        "{}:{} bitfield received ({} pieces)",
                        self.ip,
                        self.port,
                        bitfield.piece_count()
                    );

                    self.bitfield = Some(bitfield.clone());

                    peer_manager
                        .lock()
                        .await
                        .update_bitfield(&self.ip, self.port, bitfield);

                    if !self.choked {
                        self.request_next_block(&mut stream, &peer_manager, &download_manager)
                            .await?;
                    }
                }

                PeerMessage::Choke => {
                    self.choked = true;

                    peer_manager
                        .lock()
                        .await
                        .set_choked(&self.ip, self.port, true);

                    println!("{}:{} choked us", self.ip, self.port);
                }

                PeerMessage::Unchoke => {
                    self.choked = false;

                    peer_manager
                        .lock()
                        .await
                        .set_choked(&self.ip, self.port, false);

                    println!("{}:{} unchoked us", self.ip, self.port);

                    self.request_next_block(&mut stream, &peer_manager, &download_manager)
                        .await?;
                }

                PeerMessage::Have(piece) => {
                    println!("{}:{} has piece {}", self.ip, self.port, piece);
                }
                PeerMessage::Piece {
                    index,
                    begin,
                    block,
                } => {
                    let result = {
                        let mut dm = download_manager.lock().await;

                        dm.mark_received(index as usize, begin);

                        dm.insert_block(index as usize, begin, block)
                    };

                    match result {
                        Ok(Some(true)) => {
                            let finalize = {
                                let mut dm = download_manager.lock().await;

                                if dm.is_complete() && !dm.finalized {
                                    dm.finalized = true;

                                    Some((dm.output_file.clone(), dm.final_file_name.clone()))
                                } else {
                                    None
                                }
                            };

                            if let Some((source_path, final_name)) = finalize {
                                std::fs::create_dir_all("torrents/done")?;

                                let destination = format!("torrents/done/{}", final_name);

                                std::fs::rename(&source_path, &destination)?;

                                println!("\nDOWNLOAD COMPLETE\n{}", destination);

                                std::process::exit(0);
                            }
                        }

                        Ok(Some(false)) => {
                            println!(
                                "{}:{} piece {} failed hash check",
                                self.ip, self.port, index
                            );
                        }

                        Ok(None) => {}

                        Err(e) => {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                format!("failed to store piece {}: {}", index, e),
                            ));
                        }
                    }

                    self.request_next_block(&mut stream, &peer_manager, &download_manager)
                        .await?;
                }

                PeerMessage::Interested => {}

                PeerMessage::NotInterested => {}

                PeerMessage::Unknown { id, .. } => {
                    println!("{}:{} unknown message {}", self.ip, self.port, id);
                }
            }
        }
    }
}
