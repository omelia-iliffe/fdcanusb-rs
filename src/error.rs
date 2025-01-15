use derive_more::derive::{Display, Error, From};

#[derive(Error, Debug, Display, From)]
pub enum TransferError {
    Write(WriteError),
    Read(ReadError),
}

#[derive(Error, Debug, Display, From)]
pub enum WriteError {
    #[display("Failed to write to port: {_0}")]
    Io(std::io::Error),
}

#[derive(Error, Debug, Display, From)]
pub enum ReadError {
    #[display("Failed to read from port: {_0}")]
    Io(std::io::Error),
    #[display("Failed to parse packet into Uft8: {_0}")]
    Uft8(std::str::Utf8Error),
    #[display("Failed to parse response: {_0}")]
    Parse(ParseError),
    #[display("Lost sync: expected {expected}, received {received}")]
    LostSync { expected: String, received: String },
}

#[derive(Error, Debug, Display, From)]
pub enum ParseError {
    #[display("Unexpected data {received}, expected {expected}")]
    UnexpectedData { expected: String, received: String },
    #[display("Unexpected EOL, expected {expected}")]
    UnexpectedEOL { expected: String },
    #[display("Unexpected data with flag {flag}, {data}")]
    UnexpectedFlagData { flag: String, data: String },
    #[display("Failed to parse ID: {_0}")]
    ID(std::num::ParseIntError),
    #[display("Failed to parse data: {_0}")]
    #[from]
    Data(hex::FromHexError),
    #[display("Failed to parse timestamp: {_0}")]
    TimeStamp(std::num::ParseIntError),
}

#[derive(Error, Debug, Display)]
#[display("Max frame length of 64 exceeded: {_0}")]
pub struct InvalidFrameLength(#[error(not(source))] pub usize);
