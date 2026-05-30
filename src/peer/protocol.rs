use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug)]
pub enum PeerMessage {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(Vec<u8>),

    Piece {
        index: u32,
        begin: u32,
        block: Vec<u8>,
    },

    Unknown {
        id: u8,
    },
}

pub fn decode_message(id: u8, payload: Vec<u8>) -> Result<PeerMessage, String> {
    match id {
        0 => Ok(PeerMessage::Choke),

        1 => Ok(PeerMessage::Unchoke),

        2 => Ok(PeerMessage::Interested),

        3 => Ok(PeerMessage::NotInterested),

        4 => {
            if payload.len() != 4 {
                return Err("invalid have message".into());
            }

            let piece = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);

            Ok(PeerMessage::Have(piece))
        }

        5 => Ok(PeerMessage::Bitfield(payload)),

        7 => {
            if payload.len() < 8 {
                return Err("invalid piece message".into());
            }

            let index = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);

            let begin = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);

            let block = payload[8..].to_vec();

            Ok(PeerMessage::Piece {
                index,
                begin,
                block,
            })
        }

        _ => Ok(PeerMessage::Unknown { id }),
    }
}

pub async fn read_message<R>(reader: &mut R) -> Result<PeerMessage, String>
where
    R: AsyncRead + Unpin,
{
    let mut length_buf = [0u8; 4];

    reader
        .read_exact(&mut length_buf)
        .await
        .map_err(|e| e.to_string())?;

    let length = u32::from_be_bytes(length_buf);

    if length == 0 {
        return Ok(PeerMessage::KeepAlive);
    }

    let mut id_buf = [0u8; 1];

    reader
        .read_exact(&mut id_buf)
        .await
        .map_err(|e| e.to_string())?;

    let id = id_buf[0];

    let payload_length = length as usize - 1;

    let mut payload = vec![0u8; payload_length];

    reader
        .read_exact(&mut payload)
        .await
        .map_err(|e| e.to_string())?;

    decode_message(id, payload)
}

pub fn interested_message() -> [u8; 5] {
    [0, 0, 0, 1, 2]
}

pub fn request_message(piece_index: u32, begin: u32, length: u32) -> Vec<u8> {
    let mut msg = Vec::with_capacity(17);

    msg.extend_from_slice(&13u32.to_be_bytes());

    msg.push(6);

    msg.extend_from_slice(&piece_index.to_be_bytes());

    msg.extend_from_slice(&begin.to_be_bytes());

    msg.extend_from_slice(&length.to_be_bytes());

    msg
}
