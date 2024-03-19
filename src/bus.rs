use serial2::IntoSettings;
use std::ops::Deref;
use std::path::Path;
use std::str;
use std::time::Duration;

use crate::frames::{CanFdFrame, FdCanUSBFrame};

/// FdCanUSB communications struct
///
/// Can be used with any transport type that implements `std::io::Write` and `std::io::Read`
/// The baud rate is unused, as the FdCanUSB communicates via USB CDC
///
/// For convenience, we provide a `FdCanUSB` implementation for `serial2::SerialPort`.
///
/// ## Example
/// ```
/// use fdcanusb::FdCanUSB;
///
/// let transport = serial2::SerialPort::open("/dev/fdcanusb", serial2::KeepSettings).expect("Failed to open serial port");
/// let mut fdcanusb = FdCanUSB::new(transport);
/// ```
#[derive(Debug)]

pub struct FdCanUSB<T>
where
    T: std::io::Write + std::io::Read,
{
    transport: T,
}

impl FdCanUSB<serial2::SerialPort> {
    /// For convenience, we provide a `FdCanUSB` implementation for `serial2::SerialPort`.
    /// ```
    /// use fdcanusb::FdCanUSB;
    ///
    /// let mut fdcanusb = FdCanUSB::open("/dev/fdcanusb", serial2::KeepSettings).expect("Failed to open serial port");
    /// ```
    pub fn open<P: AsRef<Path>>(
        path: P,
        serial_settings: impl IntoSettings,
    ) -> std::io::Result<Self> {
        let mut transport = serial2::SerialPort::open(path, serial_settings)?;
        transport.set_read_timeout(Duration::from_millis(100))?;
        Ok(Self::new(transport))
    }
}

impl<T> FdCanUSB<T>
where
    T: std::io::Write + std::io::Read,
{
    /// Create a new FdCanUSB instance
    pub fn new(transport: T) -> Self {
        FdCanUSB { transport }
    }

    /// Transfer a single frame.
    /// If `response` is `true`, the function will wait for a response frame.
    /// Otherwise, it will return `None`.
    pub fn transfer_single(
        &mut self,
        frame: CanFdFrame,
        response: bool,
    ) -> std::io::Result<Option<CanFdFrame>> {
        let frame: FdCanUSBFrame = frame.into();
        self.write_frame(frame)?;
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
    pub fn write_frame(&mut self, frame: FdCanUSBFrame) -> std::io::Result<()> {
        log::trace!("> {:?}", frame);
        self.transport.write_all(frame.as_bytes())?;
        Ok(())
    }

    /// The FdCanUSB responds with `OK` after a correct frame is parsed.
    /// `read_ok` waits for this response, and returns an error if it is not received.
    pub fn read_ok(&mut self) -> std::io::Result<()> {
        let mut buffer = [0; 4];
        let read_num = self.transport.read(&mut buffer)?;
        match buffer.starts_with(b"OK") {
            true => Ok(()),
            false => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Lost sync (expected OK, got: {:?})",
                    String::from_utf8_lossy(&buffer[..read_num])
                ),
            )),
        }
    }

    /// Read a response frame from the FdCanUSB.
    pub fn read_response(&mut self) -> std::io::Result<CanFdFrame> {
        let mut buffer = [0; 200];
        let read_num = self.transport.read(&mut buffer)?;
        let response = str::from_utf8(&buffer[..read_num]).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse response: {}", e),
            )
        })?;
        log::trace!("< {:?}", response);
        let response = FdCanUSBFrame::from(response);
        response.try_into()
    }

    /// Flush the FdCanUSB.
    /// This can be important to do when initializing the FdCanUSB, as any data in the buffer can cause lost sync issues.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.transport.flush()
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
