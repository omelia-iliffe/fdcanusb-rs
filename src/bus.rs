use serial2::IntoSettings;
use std::ops::Deref;
use std::path::Path;
use std::time::Duration;

use crate::frames::{CanFdFrame, FdCanUSBFrame};

pub struct FdCanUSB<T>
where
    T: std::io::Write + std::io::Read,
{
    transport: T,
}

impl FdCanUSB<serial2::SerialPort> {
    pub fn new<P: AsRef<Path>>(path: P, settings: impl IntoSettings) -> std::io::Result<Self> {
        let mut transport = serial2::SerialPort::open(path, settings)?;
        transport.set_read_timeout(Duration::from_millis(100))?;
        Ok(FdCanUSB { transport })
    }
}

impl<T> FdCanUSB<T>
where
    T: std::io::Write + std::io::Read,
{
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

    pub fn write_frame(&mut self, frame: FdCanUSBFrame) -> std::io::Result<()> {
        log::trace!("> {:?}", frame);
        self.transport.write_all(frame.as_bytes())?;
        Ok(())
    }
    pub fn read_ok(&mut self) -> std::io::Result<()> {
        let mut buffer = [0; 4];
        let read_num = self.transport.read(&mut buffer).unwrap();
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
    pub fn read_response(&mut self) -> std::io::Result<CanFdFrame> {
        let mut buffer = [0; 200];
        let read_num = self.transport.read(&mut buffer).unwrap();
        let response = String::from_utf8_lossy(&buffer[..read_num]);
        log::trace!("< {:?}", response);
        let response = FdCanUSBFrame::from(response.deref());
        response.try_into()
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.transport.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fdcanusb() {
        let mut fdcanusb = FdCanUSB::new("/dev/fdcanusb", 9600).unwrap();
        let frame = FdCanUSBFrame::from(
            "can send 8001 01000A0D200000C07F0D270000004011001F01130D505050 b\n",
        );
        // let frame= FdCanUSBFrame::from("can send 8001 01000011001F01130D505050 b");

        fdcanusb.write_frame(frame).unwrap();
        fdcanusb.read_ok().unwrap();
        let respsonse = fdcanusb.read_response();
        dbg!(&respsonse);
        assert!(respsonse.is_ok());
    }
}
