//! re-export embedded_can ID types and provide helper functions for encoding/decoding Id's from be byte slices

pub use embedded_can::{ExtendedId, Id, StandardId};

fn pad_bytes<const N: usize>(bytes: &[u8]) -> Option<[u8; N]> {
    if bytes.len() > N {
        return None;
    }
    let mut arr = [0u8; N];
    let len = bytes.len().min(N);
    arr[N - len..].copy_from_slice(&bytes[bytes.len() - len..]);
    Some(arr)
}

pub(crate) fn decode_standerd_id(bytes: &[u8]) -> Option<StandardId> {
    let bytes = pad_bytes(bytes)?;
    let id = u16::from_be_bytes(bytes);
    StandardId::new(id)
}

pub(crate) fn decode_extended_id(bytes: &[u8]) -> Option<ExtendedId> {
    let bytes = pad_bytes(bytes)?;
    let id = u32::from_be_bytes(bytes);
    ExtendedId::new(id)
}

pub fn encode_id(id: Id) -> String {
    let id = match id {
        Id::Standard(id) => hex::encode_upper(id.as_raw().to_be_bytes()),
        Id::Extended(id) => hex::encode_upper(id.as_raw().to_be_bytes()),
    };

    id.trim_start_matches('0').into()
}
