use crate::{
    InvalidFrameLength,
    error::ParseError,
    id::{Id, decode_extended_id, decode_standerd_id, encode_id},
};

fn decode_hex(s: &str) -> Result<Vec<u8>, hex::FromHexError> {
    if s.len() % 2 == 1 {
        hex::decode(format!("0{}", s))
    } else {
        hex::decode(s)
    }
}

/// `CanFdFrame` represents a single frame of data on the CAN bus.
#[derive(Debug)]
pub struct CanFrame {
    /// The arbitration id of the frame.
    pub id: Id,
    /// The data of the frame, up to 8 bytes.
    pub data: Vec<u8>,
    /// If the frame is a remote frame.
    pub remote_frame: Option<bool>,
    /// The timestamp of the frame.
    pub timestamp: Option<u32>,
}

/// `CanFdFrame` represents a single frame of data on the CAN bus.
#[derive(Debug)]
pub struct CanFdFrame {
    /// The arbitration id of the frame.
    pub id: Id,
    /// The data of the frame, up to 64 bytes.
    pub data: Vec<u8>,
    /// If the frame has bit rate switching.
    pub brs: Option<bool>,
    /// If the frame is a remote frame.
    pub remote_frame: Option<bool>,
    /// The timestamp of the frame.
    pub timestamp: Option<u32>,
}

impl CanFrame {
    /// Create a new `CanFdFrame` with the given arbitration id and data.
    /// returns an `Err` if the length of `data` is > 64
    ///
    /// Use [`CanFdFrame::with_brs`], [`CanFdFrame::with_timestamp`], and [`CanFdFrame::with_remote_frame`] to set the flags.
    pub fn new(arbitration_id: impl Into<Id>, data: Vec<u8>) -> Result<Self, InvalidFrameLength> {
        if data.len() > 8 {
            return Err(InvalidFrameLength(data.len()));
        }
        Ok(Self {
            id: arbitration_id.into(),
            data,
            remote_frame: None,
            timestamp: None,
        })
    }

    pub fn with_remote_frame(mut self, remote_frame: bool) -> Self {
        self.remote_frame = Some(remote_frame);
        self
    }
}

impl CanFdFrame {
    /// Create a new `CanFdFrame` with the given arbitration id and data.
    /// returns an `Err` if the length of `data` is > 64
    ///
    /// Use [`CanFdFrame::with_brs`], [`CanFdFrame::with_timestamp`], and [`CanFdFrame::with_remote_frame`] to set the flags.
    pub fn new(arbitration_id: impl Into<Id>, data: Vec<u8>) -> Result<Self, InvalidFrameLength> {
        if data.len() > 64 {
            return Err(InvalidFrameLength(data.len()));
        }
        Ok(Self {
            id: arbitration_id.into(),
            data,
            brs: None,
            remote_frame: None,
            timestamp: None,
        })
    }

    pub fn with_brs(mut self, brs: bool) -> Self {
        self.brs = Some(brs);
        self
    }

    pub fn with_remote_frame(mut self, remote_frame: bool) -> Self {
        self.remote_frame = Some(remote_frame);
        self
    }
}

/// The FdCanUSB communicates over Serial using ascii encoded frames.
/// `FdCanUSBFrame` is a wrapper around the ascii encoded frames.
///
/// They can be converted to and from `CanFdFrame`
#[derive(Debug)]
pub struct FdCanUSBFrame(String);

impl From<&str> for FdCanUSBFrame {
    /// Create a `FdCanUSBFrame` from a string.
    /// **No** validation is performed to check if it is a valid frame.
    /// The frame will be validated when converting to a `CanFdFrame`
    fn from(data: &str) -> FdCanUSBFrame {
        FdCanUSBFrame(data.to_owned())
    }
}

impl FdCanUSBFrame {
    pub(crate) fn new(
        id: String,
        data: impl AsRef<[u8]>,
        brs: Option<bool>,
        fd_can_frame: Option<bool>,
        remote_frame: Option<bool>,
    ) -> Self {
        let data = data.as_ref();
        let num_bytes = data.len();
        let data = hex::encode_upper(data);
        let data_len = {
            match num_bytes {
                ..=8 => num_bytes,
                9..=12 => 12,
                13..=16 => 16,
                17..=20 => 20,
                21..=24 => 24,
                25..=32 => 32,
                33..=48 => 48,
                49..=64 => 64,
                _ => panic!("Invalid data length {num_bytes}"),
            }
        };
        let padding_len = data_len - num_bytes; // data_len will always be equal or greater than num_bytes
        let padding: String = (0..padding_len).map(|_| "50").collect();
        let data = format!("{data}{padding}");
        let flags = {
            let mut flags = String::new();
            match brs {
                Some(true) => flags.push_str(" B"),
                Some(false) => flags.push_str(" b"),
                None => {}
            }
            match fd_can_frame {
                Some(true) => flags.push_str(" F"),
                Some(false) => flags.push_str(" f"),
                None => {}
            }
            match remote_frame {
                Some(true) => flags.push_str(" R"),
                Some(false) => flags.push_str(" r"),
                None => {}
            }
            flags
        };
        Self(format!("can send {id} {data}{flags}\n"))
    }

    /// Get the ascii encoded frame as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    fn parse(self) -> Result<FdCanUsbFrameParse, ParseError> {
        let mut iter = self.0.trim().split(' ');
        match iter.next() {
            Some("rcv") => {}
            Some(unexpected) => {
                return Err(ParseError::UnexpectedData {
                    expected: "rcv".to_string(),
                    received: unexpected.to_string(),
                });
            }
            None => {
                return Err(ParseError::UnexpectedEOL {
                    expected: "rcv".to_string(),
                });
            }
        };

        let id = iter.next().ok_or_else(|| ParseError::UnexpectedEOL {
            expected: "id".to_string(),
        })?;

        let data = iter.next().ok_or_else(|| ParseError::UnexpectedEOL {
            expected: "data".to_string(),
        })?;

        let mut frame = FdCanUsbFrameParse {
            arbitration_id: decode_hex(id)?,
            data: decode_hex(data)?,
            ..Default::default()
        };

        // E/e frame was received with extended/classic ID
        // B/b frame was received with/without bitrate switching
        // F/f frame was received in fdcan/classic mode
        // R/r frame was remote/data frame
        // tNNNNN timestamp of receipt measured in microseconds
        // fNN integer ID of which filter matched this frame
        iter.try_for_each(|flag| {
            match flag {
                "E" => frame.extended_id = true,
                "B" => frame.brs = true,
                "F" => frame.fd_can_frame = true,
                "R" => frame.remote_frame = true,
                t if flag.starts_with('t') => {
                    frame.timestamp = Some(
                        t.strip_prefix('t')
                            .expect("starts with t checked")
                            .parse()?,
                    )
                }
                "f-1" => (), // no filter matched
                f if f.starts_with('f') && flag.len() > 1 => {
                    // todo impl filter parsing
                }
                flag if flag.to_lowercase() == flag => (),
                flag => panic!("unknown flag {flag}"),
            };
            Ok::<(), ParseError>(())
        })?;

        // let arbitration_id = Id::decode_hex(id).ok_or(ParseError::InvalidId(id.to_string()))?;

        Ok(frame)
    }
}

#[derive(Debug, Default)]
struct FdCanUsbFrameParse {
    arbitration_id: Vec<u8>,
    data: Vec<u8>,
    extended_id: bool,
    brs: bool,
    fd_can_frame: bool,
    remote_frame: bool,
    timestamp: Option<u32>,
    filter: Option<u32>,
}

impl From<&CanFrame> for FdCanUSBFrame {
    fn from(frame: &CanFrame) -> FdCanUSBFrame {
        let id = encode_id(frame.id);
        FdCanUSBFrame::new(id, &frame.data, None, Some(false), frame.remote_frame)
    }
}

impl TryFrom<FdCanUSBFrame> for CanFrame {
    type Error = ParseError;
    fn try_from(data: FdCanUSBFrame) -> Result<Self, Self::Error> {
        let FdCanUsbFrameParse {
            arbitration_id,
            data,
            extended_id,
            brs: _,
            fd_can_frame,
            remote_frame,
            timestamp,
            filter: _,
        } = data.parse()?;

        if fd_can_frame {
            return Err(ParseError::UnexpectedFdFrame);
        }
        let id = if extended_id {
            decode_extended_id(&arbitration_id)
                .ok_or(ParseError::InvalidId(hex::encode(arbitration_id)))?
                .into()
        } else {
            decode_standerd_id(&arbitration_id)
                .ok_or(ParseError::InvalidId(hex::encode(arbitration_id)))?
                .into()
        };
        Ok(Self {
            id,
            data,
            remote_frame: Some(remote_frame),
            timestamp,
        })
    }
}

impl From<&CanFdFrame> for FdCanUSBFrame {
    fn from(frame: &CanFdFrame) -> FdCanUSBFrame {
        let id = encode_id(frame.id);
        FdCanUSBFrame::new(id, &frame.data, frame.brs, Some(true), frame.remote_frame)
    }
}

impl TryFrom<FdCanUSBFrame> for CanFdFrame {
    type Error = ParseError;
    fn try_from(data: FdCanUSBFrame) -> Result<Self, Self::Error> {
        let FdCanUsbFrameParse {
            arbitration_id,
            data,
            extended_id,
            brs,
            fd_can_frame,
            remote_frame,
            timestamp,
            filter: _,
        } = data.parse()?;

        if !fd_can_frame {
            return Err(ParseError::UnexpectedClassicFrame);
        }
        let id = if extended_id {
            decode_extended_id(&arbitration_id)
                .ok_or(ParseError::InvalidId(hex::encode(arbitration_id)))?
                .into()
        } else {
            decode_standerd_id(&arbitration_id)
                .ok_or(ParseError::InvalidId(hex::encode(arbitration_id)))?
                .into()
        };
        Ok(Self {
            id,
            data,
            brs: Some(brs),
            remote_frame: Some(remote_frame),
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::id::{ExtendedId, StandardId};

    use super::*;

    #[test]
    fn test_can_fd_frame_encode() {
        let frame = CanFdFrame::new(
            ExtendedId::new(0x8001).unwrap(),
            vec![
                1, 0, 10, 13, 32, 0, 0, 192, 127, 13, 39, 0, 0, 0, 64, 17, 0, 31, 1, 19, 13,
            ],
        )
        .unwrap();
        let encode_frame: FdCanUSBFrame = (&frame).into();
        assert_eq!(
            encode_frame.0,
            "can send 8001 01000A0D200000C07F0D270000004011001F01130D505050 F\n".to_owned()
        );
    }

    #[test]
    fn test_can_fd_frame_decode() {
        let frame = FdCanUSBFrame(
            "rcv 8001 01000A0D200000C07F0D270000004011001F01130D505050 E b F r f-1\n".to_owned(),
        );
        let decode_frame: CanFdFrame = frame.try_into().expect("Failed to decode frame");
        assert_eq!(decode_frame.id, ExtendedId::new(0x8001).unwrap().into());
        assert_eq!(
            decode_frame.data,
            vec![
                0x01, 0x00, 0x0A, 0x0D, 0x20, 0x00, 0x00, 0xC0, 0x7F, 0x0D, 0x27, 0x00, 0x00, 0x00,
                0x40, 0x11, 0x00, 0x1F, 0x01, 0x13, 0x0D, 0x50, 0x50, 0x50,
            ]
        );
    }

    #[test]
    fn test_can_fd_frame_flags_encode() {
        let frame = CanFdFrame::new(
            ExtendedId::new(0x8001).unwrap(),
            vec![
                1, 0, 10, 13, 32, 0, 0, 192, 127, 13, 39, 0, 0, 0, 64, 17, 0, 31, 1, 19, 13,
            ],
        )
        .unwrap()
        .with_brs(true);
        let encode_frame: FdCanUSBFrame = (&frame).into();
        assert_eq!(
            encode_frame.0,
            "can send 8001 01000A0D200000C07F0D270000004011001F01130D505050 B F\n".to_owned()
        );
    }

    #[test]
    fn test_can_frame_encode() {
        let frame = CanFrame::new(
            StandardId::new(1).unwrap(),
            vec![1, 0, 10, 13, 32, 0, 0, 192],
        )
        .unwrap();
        let encode_frame: FdCanUSBFrame = (&frame).into();
        assert_eq!(encode_frame.0, "can send 1 01000A0D200000C0 f\n".to_owned());
    }

    #[test]
    fn test_can_frame_decode() {
        let frame = FdCanUSBFrame("rcv 1 01000A0D200000C0 e b f r f-1\n".to_owned());
        let decode_frame: CanFrame = frame.try_into().expect("Failed to decode frame");
        assert_eq!(decode_frame.id, StandardId::new(1).unwrap().into());
        assert_eq!(
            decode_frame.data,
            vec![0x01, 0x00, 0x0A, 0x0D, 0x20, 0x00, 0x00, 0xC0]
        );
    }
}
