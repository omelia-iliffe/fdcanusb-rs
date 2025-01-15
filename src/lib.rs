//! # FdCanUSB
//! An implementation of the [FdCanUSB](https://mjbots.com/products/fdcanusb) (by [MJBots](https://mjbots.com/)) protocol.
//!
//! ### Example
//! ```
//! use fdcanusb::{FdCanUSB, serial2};
//! # fn main() -> Result<(), std::io::Error> {
//! let mut fdcanusb = FdCanUSB::open("/dev/fdcanusb");
//! # Ok(())
//! # }
//! ```

#[macro_use]
mod log;
mod bus;
mod error;
mod frames;

pub use bus::FdCanUSB;
pub use error::*;
pub use frames::{CanFdFrame, FdCanUSBFrame};

pub use serial2;
