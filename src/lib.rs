//! # FdCanUSB
//! An implementation of the [FdCanUSB](https://mjbots.com/products/fdcanusb) (by [MJBots](https://mjbots.com/)) protocol.
//!
//! This initial release is open for feedback and may change implementation details.
//! ### Example
//! ```
//! use fdcanusb::{FdCanUSB, serial2};
//! # fn main() -> Result<(), std::io::Error> {
//! let transport = serial2::SerialPort::open("/dev/fdcanusb", serial2::KeepSettings)?;
//! let mut fdcanusb = FdCanUSB::new(transport);
//! # Ok(())
//! # }
//! ```
//!
//! ### Features
//!
//! - `default = ["serial2"]`
//! - `serial2`
//!     - Enables re-exporting of the [`serial2`] crate and the [`FdCanUSB::open`] fn.

#[macro_use]
mod log;
mod bus;
mod error;
mod frames;

pub use bus::FdCanUSB;
pub use error::*;
pub use frames::{CanFdFrame, FdCanUSBFrame};

#[cfg(feature = "serial2")]
pub use serial2;
