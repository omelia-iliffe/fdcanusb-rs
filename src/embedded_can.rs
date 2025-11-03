use embedded_can::{Error, Frame, blocking::Can};

use crate::{
    CanFrame, TransferError,
    id::{ExtendedId, Id, StandardId},
};

impl From<StandardId> for embedded_can::StandardId {
    fn from(value: StandardId) -> Self {
        embedded_can::StandardId::new(value.as_raw()).unwrap()
    }
}

impl From<embedded_can::StandardId> for StandardId {
    fn from(value: embedded_can::StandardId) -> Self {
        StandardId::new(value.as_raw()).unwrap()
    }
}

impl From<ExtendedId> for embedded_can::ExtendedId {
    fn from(value: ExtendedId) -> Self {
        embedded_can::ExtendedId::new(value.as_raw()).unwrap()
    }
}

impl From<embedded_can::ExtendedId> for ExtendedId {
    fn from(value: embedded_can::ExtendedId) -> Self {
        ExtendedId::new(value.as_raw()).unwrap()
    }
}

impl From<Id> for embedded_can::Id {
    fn from(value: Id) -> Self {
        match value {
            Id::Standard(id) => embedded_can::Id::Standard(id.into()),
            Id::Extended(id) => embedded_can::Id::Extended(id.into()),
        }
    }
}

impl From<embedded_can::Id> for Id {
    fn from(value: embedded_can::Id) -> Self {
        match value {
            embedded_can::Id::Standard(id) => Id::Standard(id.into()),
            embedded_can::Id::Extended(id) => Id::Extended(id.into()),
        }
    }
}

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
        self.id.into()
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
