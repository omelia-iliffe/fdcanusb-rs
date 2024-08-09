use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransferError {
    #[error(transparent)]
    Write(#[from] WriteError),
    #[error(transparent)]
    Read(#[from] ReadError),
}

#[derive(Error, Debug)]
pub enum WriteError {
    #[error("Failed to write to port: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Failed to read from port: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse packet into Uft8: {0}")]
    Uft8(#[from] std::str::Utf8Error),
    #[error("Failed to parse response: {0}")]
    Parse(#[from] ParseError),
    #[error("Lost sync: expected {expected}, received {received}")]
    LostSync { expected: String, received: String },
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected data {received}, expected {expected}")]
    UnexpectedData { expected: String, received: String },
    #[error("Unexpected EOL, expected {expected}")]
    UnexpectedEOL { expected: String },
    #[error("Unexpected data with flag {flag}, {data}")]
    UnexpectedFlagData { flag: String, data: String },
    #[error("Failed to parse ID: {0}")]
    ID(std::num::ParseIntError),
    #[error("Failed to parse data: {0}")]
    Data(#[from] hex::FromHexError),
    #[error("Failed to parse timestamp: {0}")]
    TimeStamp(std::num::ParseIntError),
}

#[derive(Error, Debug)]
#[error("Max frame length of 64 exceeded: {0}")]
pub struct InvalidFrameLength(pub usize);
