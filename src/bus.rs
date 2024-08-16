use crate::error::{ReadError, TransferError, WriteError};
use crate::frames::{CanFdFrame, FdCanUSBFrame};

/// FdCanUSB communications struct
///
/// Can be used with any transport type that implements [`std::io::Write`] and [`std::io::Read`]
/// The baud rate is unused, as the [FdCanUSB] communicates via USB CDC
/// ### Example
/// ```
/// use fdcanusb::{FdCanUSB, serial2};
/// # fn main() -> Result<(), std::io::Error> {
/// let transport = serial2::SerialPort::open("/dev/fdcanusb", serial2::KeepSettings)?;
/// let mut fdcanusb = FdCanUSB::new(transport);
/// # Ok(())
/// # }
/// ```
/// ## `serial2` Integration
/// To use the `FdCanUSB` with a [`serial2::SerialPort`](https://docs.rs/serial2/latest/serial2/), you can use the [`FdCanUSB::open`] method.
/// ### Example
/// ```
/// use fdcanusb::FdCanUSB;
///
/// let mut fdcanusb = FdCanUSB::open("/dev/fdcanusb", serial2::KeepSettings).expect("Failed to open serial port");
/// ```

#[derive(Debug)]

pub struct FdCanUSB<T, Buffer = Vec<u8>>
where
    T: std::io::Write + std::io::Read,
    Buffer: AsRef<[u8]> + AsMut<[u8]>,
{
    /// The transport used the communicate with the FdCanUSB
    transport: T,
    /// The buffer used to store data read from the FdCanUSB
    buffer: Buffer,
    /// The total number of valid bytes in the buffer
    read_len: usize,
    /// The number of leading bytes in the buffer that have already been used
    used_bytes: usize,
}

#[cfg(feature = "serial2")]
impl FdCanUSB<serial2::SerialPort, Vec<u8>> {
    /// For convenience, we provide a [`FdCanUSB`] implementation for [`serial2::SerialPort`].
    /// Enable with the `serial2` feature.

    pub fn open<P: AsRef<std::path::Path>>(
        path: P,
        serial_settings: impl serial2::IntoSettings,
    ) -> std::io::Result<Self> {
        let mut transport = serial2::SerialPort::open(path, serial_settings)?;
        transport.set_read_timeout(std::time::Duration::from_millis(100))?;
        transport.flush()?;
        transport.discard_buffers()?;
        Ok(Self::new(transport))
    }

    /// Flush the FdCanUSB.
    /// This can be important to do when initializing the FdCanUSB, as any data in the buffer can cause lost sync issues.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.transport.flush()?;
        self.transport.discard_buffers()
    }
}


impl<T> FdCanUSB<T, Vec<u8>>
where
    T: std::io::Write + std::io::Read,
{
    /// Create a new [FdCanUSB] instance
    pub fn new(transport: T) -> Self {
        Self::new_with_buffer(transport, vec![0; 256])
    }
}

impl<T, Buffer> FdCanUSB<T, Buffer>
where
    T: std::io::Write + std::io::Read,
    Buffer: AsRef<[u8]> + AsMut<[u8]>,
{
    /// Create a new [FdCanUSB] instance, with a given transport and buffer.
    pub fn new_with_buffer(transport: T, buffer: Buffer) -> Self {
        FdCanUSB { transport, buffer, read_len: 0, used_bytes: 0 }
    }

    /// Transfer a single frame.
    /// If `response` is `true`, the function will wait for a response frame.
    /// Otherwise, it will return `None`.
    pub fn transfer_single(
        &mut self,
        frame: CanFdFrame,
        response: bool,
    ) -> Result<Option<CanFdFrame>, TransferError> {
        let frame: FdCanUSBFrame = frame.into();
        self.write_frame(frame)?;
        self.read_len = 0;
        self.used_bytes = 0;
        self.read_ok()?;
        if response {
            Ok(Some(self.read_response()?))
        } else {
            Ok(None)
        }
    }

    /// Write a frame to the FdCanUSB
    ///
    /// Frames are logged at the `trace` level by default.
    fn write_frame(&mut self, frame: FdCanUSBFrame) -> Result<(), WriteError> {
        log::debug!("> {:?}", frame);
        self.transport.write_all(frame.as_bytes())?;
        Ok(())
    }

    /// Reads bytes into the buffer and returns the end pos of one packet.
    /// Packets are seperated by `/r/n`.
    fn read_newline(&mut self) -> Result<usize, std::io::Error> {
        let buffer = self.buffer.as_mut();
        let timeout = std::time::Instant::now() + std::time::Duration::from_millis(100);
        loop {
            if let Some(pos) = buffer[self.used_bytes..self.read_len].iter().position(|&c| c == b'\n') {
                log::trace!("raw packet {:?}", &buffer[self.used_bytes..self.used_bytes + pos + 1]);
                return Ok(self.used_bytes + pos + 1);
            }
            if std::time::Instant::now() > timeout {
                return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Timed out waiting for newline"));
            }
            let read_num = self.transport.read(&mut buffer[self.read_len..])?;
            log::trace!("read {} {:?}", read_num, &buffer[self.read_len..self.read_len + read_num]);
            self.read_len += read_num;
        }
    }

    /// The [FdCanUSB] responds with `OK` after a correct frame is parsed.
    /// `read_ok` waits for this response, and returns an error if it is not received.
    fn read_ok(&mut self) -> Result<(), ReadError> {
        let packet = self.read_newline()?;
        let packet= &self.buffer.as_ref()[self.used_bytes..packet];
        self.used_bytes += packet.len();
        if packet.starts_with(b"OK") {
            Ok(())
        } else {
            Err(ReadError::LostSync {
                expected: "OK".to_string(),
                received: String::from_utf8_lossy(packet).to_string(),
            })
        }
    }

    /// Read a response frame from the [FdCanUSB].
    /// Responses are logged at the `trace` level by default.
    fn read_response(&mut self) -> Result<CanFdFrame, ReadError> {
        let packet = self.read_newline()?;
        let packet= &self.buffer.as_ref()[self.used_bytes..packet];
        self.used_bytes += packet.len();
        if packet.starts_with(b"rcv") {
            let response = std::str::from_utf8(packet)?;
            log::debug!("< {:?}", response);
            let response = FdCanUSBFrame::from(response);
            let response = response.try_into()?;
            Ok(response)
        } else {
            Err(ReadError::LostSync {
                expected: "rcv".to_string(),
                received: String::from_utf8_lossy(packet).to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fdcanusb() {
        let mut fdcanusb = FdCanUSB::open("/dev/fdcanusb", serial2::KeepSettings)
            .expect("Failed to open fdcanusb");
        let frame = FdCanUSBFrame::from(
            "can send 8001 01000A0D200000C07F0D270000004011001F01130D505050 b\n",
        );
        // let frame= FdCanUSBFrame::from("can send 8001 01000011001F01130D505050 b");

        fdcanusb.write_frame(frame).expect("Failed to write frame");
        fdcanusb.read_ok().expect("Failed to read ok");
        let respsonse = fdcanusb.read_response();
        dbg!(&respsonse);
        assert!(respsonse.is_ok());
    }
}
