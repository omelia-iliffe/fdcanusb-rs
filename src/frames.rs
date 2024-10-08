use crate::error::{InvalidFrameLength, ParseError};

/// `CanFdFrame` represents a single frame of data on the CAN bus.
#[derive(Debug, Default)]
pub struct CanFdFrame {
    /// The arbitration id of the frame.
    ///
    // TODO: Integrate with the `CanId` type from the `socketcan` crate? or reimplement it?
    pub arbitration_id: u16,
    /// The data of the frame, up to 64 bytes.
    pub data: Vec<u8>,
    /// If the frame has an extended id.
    pub extended_id: Option<bool>,
    /// If the frame has bit rate switching.
    pub brs: Option<bool>,
    /// If the frame is a CAN FD frame.
    pub fd_can_frame: Option<bool>,
    /// If the frame is a remote frame.
    pub remote_frame: Option<bool>,
    /// The timestamp of the frame.
    pub timestamp: Option<u32>,
}

impl CanFdFrame {
    /// Create a new `CanFdFrame` with the given arbitration id and data.
    /// returns an `Err` if the length of `data` is > 64
    ///
    /// Use [`CanFdFrame::new_with_flags`] to set the flags.
    pub fn new(arbitration_id: u16, data: &[u8]) -> Result<CanFdFrame, InvalidFrameLength> {
        if data.len() > 64 {
            return Err(InvalidFrameLength(data.len()));
        }
        Ok(CanFdFrame {
            arbitration_id,
            data: data.to_owned(),
            ..Default::default()
        })
    }

    /// Create a new `CanFdFrame` with the given arbitration id, data and flags.
    /// returns an `Err` if the length of `data` is > 64
    pub fn new_with_flags(
        arbitration_id: u16,
        data: &[u8],
        extended_id: Option<bool>,
        brs: Option<bool>,
        fd_can_frame: Option<bool>,
        remote_frame: Option<bool>,
        timestamp: Option<u32>,
    ) -> Result<CanFdFrame, InvalidFrameLength> {
        if data.len() > 64 {
            return Err(InvalidFrameLength(data.len()));
        }
        Ok(CanFdFrame {
            arbitration_id,
            data: data.to_owned(),
            extended_id,
            brs,
            fd_can_frame,
            remote_frame,
            timestamp,
        })
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
    /// Get the ascii encoded frame as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

// TODO: change to use `TryFrom`
impl From<CanFdFrame> for FdCanUSBFrame {
    fn from(frame: CanFdFrame) -> FdCanUSBFrame {
        let id = hex::encode_upper(frame.arbitration_id.to_be_bytes());
        let num_bytes = frame.data.len();
        let data = hex::encode_upper(frame.data);
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
            match frame.brs {
                Some(true) => flags.push_str(" B"),
                Some(false) => flags.push_str(" b"),
                None => {}
            }
            match frame.fd_can_frame {
                Some(true) => flags.push_str(" F"),
                Some(false) => flags.push_str(" f"),
                None => {}
            }
            match frame.remote_frame {
                Some(true) => flags.push_str(" R"),
                Some(false) => flags.push_str(" r"),
                None => {}
            }
            flags
        };
        FdCanUSBFrame(format!("can send {id} {data}{flags}\n"))
    }
}

impl TryFrom<FdCanUSBFrame> for CanFdFrame {
    type Error = ParseError;
    fn try_from(data: FdCanUSBFrame) -> Result<Self, Self::Error> {
        let mut iter = data.0.trim().split(' ');
        match iter.next() {
            Some("rcv") => {}
            Some(unexpected) => {
                return Err(ParseError::UnexpectedData {
                    expected: "rcv".to_string(),
                    received: unexpected.to_string(),
                })
            }
            None => {
                return Err(ParseError::UnexpectedEOL {
                    expected: "rcv".to_string(),
                })
            }
        };

        let id = iter.next().ok_or_else(|| ParseError::UnexpectedEOL {
            expected: "id".to_string(),
        })?;

        let data = iter.next().ok_or_else(|| ParseError::UnexpectedEOL {
            expected: "data".to_string(),
        })?;

        let flags: Vec<&str> = iter.collect();

        let arbitration_id = u16::from_str_radix(id, 16).map_err(ParseError::ID)?;

        let data = hex::decode(data)?;

        // E/e frame was received with extended/classic ID
        // B/b frame was received with/without bitrate switching
        // F/f frame was received in fdcan/classic mode
        // R/r frame was remote/data frame
        // tNNNNN timestamp of receipt measured in microseconds
        // fNN integer ID of which filter matched this frame
        // The flag &str pass here should be a lower case character
        let check_flag = |c: &str| -> (Option<bool>, Option<&str>) {
            flags
                .iter()
                .find(|x| x.to_lowercase().starts_with(c))
                .map_or((None, None), |x| {
                    (
                        Some(!x.starts_with(c)), // this is inverted as we want the UPPERCASE flag to be true
                        x.strip_prefix(c).filter(|x| !x.is_empty()),
                    )
                })
        };
        // The flag &str pass here should be a lower case character
        let check_flag_no_data = |c: &str| -> Result<Option<bool>, ParseError> {
            let (flag, data) = check_flag(c);
            if let Some(data) = data {
                return Err(ParseError::UnexpectedFlagData {
                    flag: c.to_string(),
                    data: data.to_string(),
                });
            }
            Ok(flag)
        };

        let extended_id = check_flag_no_data("e")?;

        let brs = check_flag_no_data("b")?;

        let fd_can_frame = check_flag_no_data("f")?;

        let remote_frame = check_flag_no_data("r")?;

        let (_, timestamp) = check_flag("t");
        let timestamp: Option<u32> = timestamp
            .map(|x| x.parse())
            .transpose()
            .map_err(ParseError::TimeStamp)?;

        // let filter_id = check_flag("f"); // will conflict with frame flag TODO: FIX

        Ok(CanFdFrame {
            arbitration_id,
            data,
            extended_id,
            brs,
            fd_can_frame,
            remote_frame,
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_fd_frame_encode() {
        let frame = CanFdFrame::new(
            0x8001,
            &[
                1, 0, 10, 13, 32, 0, 0, 192, 127, 13, 39, 0, 0, 0, 64, 17, 0, 31, 1, 19, 13,
            ],
        )
        .unwrap();
        let encode_frame: FdCanUSBFrame = frame.into();
        assert_eq!(
            encode_frame.0,
            "can send 8001 01000A0D200000C07F0D270000004011001F01130D505050\n".to_owned()
        );
    }

    #[test]
    fn test_can_fd_frame_decode() {
        let frame =
            FdCanUSBFrame("rcv 8001 01000A0D200000C07F0D270000004011001F01130D505050\n".to_owned());
        let decode_frame: CanFdFrame = frame.try_into().expect("Failed to decode frame");
        assert_eq!(decode_frame.arbitration_id, 0x8001);
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
        let frame = CanFdFrame::new_with_flags(
            0x8001,
            &[
                1, 0, 10, 13, 32, 0, 0, 192, 127, 13, 39, 0, 0, 0, 64, 17, 0, 31, 1, 19, 13,
            ],
            None,
            Some(true),
            None,
            None,
            None,
        )
        .unwrap();
        let encode_frame: FdCanUSBFrame = frame.into();
        assert_eq!(
            encode_frame.0,
            "can send 8001 01000A0D200000C07F0D270000004011001F01130D505050 B\n".to_owned()
        );
    }

    #[test]
    fn test_can_fd_frame_flags_decode() {
        let frame = FdCanUSBFrame(
            "rcv 8001 01000A0D200000C07F0D270000004011001F01130D505050 e B F r f-1 t0100"
                .to_owned(),
        );
        let decode_frame: CanFdFrame = frame.try_into().expect("Failed to decode frame");
        assert_eq!(decode_frame.arbitration_id, 0x8001);
        assert_eq!(
            decode_frame.data,
            vec![
                1, 0, 10, 13, 32, 0, 0, 192, 127, 13, 39, 0, 0, 0, 64, 17, 0, 31, 1, 19, 13, 80,
                80, 80,
            ]
        );
        assert_eq!(decode_frame.brs, Some(true));
    }
}
