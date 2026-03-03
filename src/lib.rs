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
#[cfg(feature = "embedded-can")]
pub mod embedded_can;
mod error;
mod frames;
pub mod id;

pub use bus::FdCanUSB;
pub use error::*;
pub use frames::{CanFdFrame, CanFrame, FdCanUSBFrame};

pub use serial2;
