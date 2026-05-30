use std::io;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

use crate::peer::{
    SharedDownloadManager,
    SharedPeerManager,
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
    pub async fn connect_and_handshake(
        mut self,
        handshake: Handshake,
        peer_manager: SharedPeerManager,
        download_manager: SharedDownloadManager,
    ) -> std::io::Result<()> {
        let addr = format!("{}:{}", self.ip, self.port);

        let mut stream = TcpStream::connect(&addr).await?;

        // -------------------------
        // Register peer
        // -------------------------
        peer_manager
            .lock()
            .await
            .add_peer(self.ip.clone(), self.port);

        // -------------------------
        // Send handshake
        // -------------------------
        stream.write_all(&handshake.build()).await?;

        // -------------------------
        // Read handshake response
        // -------------------------
        let mut resp = [0u8; 68];
        stream.read_exact(&mut resp).await?;

        if resp[0] != 19 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid handshake",
            ));
        }

        if &resp[28..48] != handshake.info_hash {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "info_hash mismatch",
            ));
        }

        println!("{}:{} handshake OK", self.ip, self.port);

        // -------------------------
        // Send Interested
        // -------------------------
        stream
            .write_all(&protocol::interested_message())
            .await?;

        // -------------------------
        // Message loop
        // -------------------------
        loop {
            let msg = protocol::read_message(&mut stream)
                .await
                .map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, e)
                })?;

            match msg {
                PeerMessage::KeepAlive => {}

                PeerMessage::Bitfield(bytes) => {
                    let bitfield = Bitfield::new(bytes);

                    let piece_count = bitfield.piece_count();

                    println!(
                        "{}:{} has {} pieces",
                        self.ip,
                        self.port,
                        piece_count
                    );

                    self.bitfield = Some(bitfield.clone());

                    // UPDATE PEER MANAGER
                    peer_manager
                        .lock()
                        .await
                        .update_bitfield(
                            &self.ip,
                            self.port,
                            bitfield,
                        );

                    // optional: check download progress context
                    let dm = download_manager.lock().await;
                    if let Some(piece) = dm.next_missing_piece() {
                        println!(
                            "next needed piece could be {}",
                            piece
                        );
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
                }

                PeerMessage::Have(piece) => {
                    println!(
                        "{}:{} has piece {}",
                        self.ip,
                        self.port,
                        piece
                    );

                    // OPTIONAL: later we will update bitfield incrementally
                }

                PeerMessage::Interested => {}

                PeerMessage::NotInterested => {}

                PeerMessage::Unknown { id, .. } => {
                    println!(
                        "{}:{} unknown message {}",
                        self.ip,
                        self.port,
                        id
                    );
                }
            }
        }
    }
}