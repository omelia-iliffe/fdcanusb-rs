use embedded_can::{Error, Frame, blocking::Can};

use crate::{CanFrame, TransferError, id::Id};

impl Can for crate::bus::FdCanUSB {
    type Frame = CanFrame;
    type Error = crate::error::TransferError;

    fn transmit(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
        self.write(frame)
    }

    fn receive(&mut self) -> Result<Self::Frame, Self::Error> {
        let frame = self.read()?;
        Ok(frame)
    }
}

impl Frame for CanFrame {
    fn new(id: impl Into<embedded_can::Id>, data: &[u8]) -> Option<Self> {
        CanFrame::new(id.into(), data.to_owned()).ok()
    }

    fn new_remote(_id: impl Into<embedded_can::Id>, _dlc: usize) -> Option<Self> {
        None
    }

    fn is_extended(&self) -> bool {
        matches!(self.id, Id::Extended(_))
    }

    fn is_remote_frame(&self) -> bool {
        false
    }

    fn id(&self) -> embedded_can::Id {
        self.id
    }

    fn dlc(&self) -> usize {
        self.data.len()
    }

    fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Error for TransferError {
    fn kind(&self) -> embedded_can::ErrorKind {
        embedded_can::ErrorKind::Other
    }
}
