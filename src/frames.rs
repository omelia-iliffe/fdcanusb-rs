#[derive(Debug, Default)]
pub struct CanFdFrame {
    pub arbitration_id: u16,
    pub data: Vec<u8>,
    pub extended_id: Option<bool>,
    pub brs: Option<bool>,
    pub fd_can_frame: Option<bool>,
    pub remote_frame: Option<bool>,
    pub timestamp: Option<u32>,
}

impl CanFdFrame {
    pub fn new(arbitration_id: u16, data: &[u8]) -> CanFdFrame {
        CanFdFrame {
            arbitration_id,
            data: data.to_owned(),
            ..Default::default()
        }
    }

    pub fn new_with_flags(
        arbitration_id: u16,
        data: &[u8],
        extended_id: Option<bool>,
        brs: Option<bool>,
        fdcan_frame: Option<bool>,
        remote_frame: Option<bool>,
        timestamp: Option<u32>,
    ) -> CanFdFrame {
        CanFdFrame {
            arbitration_id,
            data: data.to_owned(),
            extended_id,
            brs,
            fd_can_frame: fdcan_frame,
            remote_frame,
            timestamp,
        }
    }
}

#[derive(Debug)]
pub struct FdCanUSBFrame(String);

impl From<&str> for FdCanUSBFrame {
    fn from(data: &str) -> FdCanUSBFrame {
        FdCanUSBFrame(data.to_owned())
    }
}

impl FdCanUSBFrame {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl From<CanFdFrame> for FdCanUSBFrame {
    fn from(frame: CanFdFrame) -> FdCanUSBFrame {
        let id = hex::encode_upper(frame.arbitration_id.to_be_bytes());

        let data = hex::encode_upper(frame.data);
        let data_len = {
            match data.len() {
                ..=8 => data.len(),
                9..=12 => 12,
                13..=16 => 16,
                17..=20 => 20,
                21..=24 => 24,
                25..=32 => 32,
                33..=48 => 48,
                49..=64 => 64,
                _ => panic!("Invalid data length"),
            }
        };
        let padding_len = (data_len - data.len()) / 2; //data_len will always be equal or greater than data.len()
        let padding: String = (0..padding_len).map(|_| "50").collect();
        let data = format!("{data}{padding}");
        let flags = {
            let mut flags = String::new();
            match frame.brs {
                Some(true) => flags.push_str(" b"),
                Some(false) => flags.push_str(" B"),
                None => {}
            }
            match frame.fd_can_frame {
                Some(true) => flags.push_str(" f"),
                Some(false) => flags.push_str(" F"),
                None => {}
            }
            match frame.remote_frame {
                Some(true) => flags.push_str(" r"),
                Some(false) => flags.push_str(" R"),
                None => {}
            }
            flags
        };
        FdCanUSBFrame(format!("can send {id} {data}{flags}\n"))
    }
}

impl TryFrom<FdCanUSBFrame> for CanFdFrame {
    type Error = std::io::Error;
    fn try_from(data: FdCanUSBFrame) -> Result<Self, Self::Error> {
        let mut iter = data.0.trim().split(' ');
        match iter.next() {
            Some("rcv") => {}
            Some(unexpected) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Expected 'rcv', Found: {:?}", unexpected),
                ));
            }
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Expected 'rcv', Found EOL",
                ));
            }
        };
        //
        // if iter.next() != Some("rcv") {
        //     return Err(std::io::Error::new(
        //         std::io::ErrorKind::InvalidData,
        //         format!("Expected 'rcv', Found: {:?}", iter.next()),
        //     ));
        // }

        let id = iter.next().ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Expected id, Found EOL",
        ))?;

        let data = iter.next().ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Expected data, Found EOL",
        ))?;

        let flags: Vec<&str> = iter.collect();

        let arbitration_id = u16::from_str_radix(id, 16).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unable to parse arbitration id: {}", id),
            )
        })?;

        let data = hex::decode(data).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Unable to decode data")
        })?;

        // E/e frame was received with extended/classic ID
        // B/b frame was received with/without bitrate switching
        // F/f frame was received in fdcan/classic mode
        // R/r frame was remote/data frame
        // tNNNNN timestamp of receipt measured in microseconds
        // fNN integer ID of which filter matched this frame
        let check_flag = |c: &str| -> (Option<bool>, Option<&str>) {
            flags
                .iter()
                .find(|x| x.to_lowercase().starts_with(c))
                .map_or((None, None), |x| {
                    (
                        Some(x.starts_with(c)),
                        x.strip_prefix(c).filter(|x| !x.is_empty()),
                    )
                })
        };

        let (extended_id, None) = check_flag("e") else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unexpected data with 'e' flag",
            ));
        };
        let (brs, None) = check_flag("b") else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unexpected data with 'b' flag",
            ));
        };
        let (fd_can_frame, None) = check_flag("f") else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unexpected data with 'f' flag",
            ));
        };
        let (remote_frame, None) = check_flag("r") else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unexpected data with 'r' flag",
            ));
        };
        let (_, timestamp) = check_flag("t");
        let timestamp: Option<u32> = match timestamp.map(|x| x.parse()) {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unable to parse timestamp data: {}", e),
                ));
            }
            None => None,
        };

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
        );
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
        let decode_frame: CanFdFrame = frame.try_into().unwrap();
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
        );
        let encode_frame: FdCanUSBFrame = frame.into();
        assert_eq!(
            encode_frame.0,
            "can send 8001 01000A0D200000C07F0D270000004011001F01130D505050 b\n".to_owned()
        );
    }

    #[test]
    fn test_can_fd_frame_flags_decode() {
        let frame = FdCanUSBFrame(
            "rcv 8001 01000A0D200000C07F0D270000004011001F01130D505050 e b F r f-1 t00100"
                .to_owned(),
        );
        let decode_frame: CanFdFrame = frame.try_into().unwrap();
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
