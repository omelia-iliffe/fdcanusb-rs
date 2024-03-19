//! # FdCanUSB
//! An implementation of the [FdCanUSB](https://mjbots.com/products/fdcanusb) (by [MJBots](https://mjbots.com/)) protocol.
//!
//! This initial release is open for feedback and may change implementation details.
//! ### Example
//! ```
//! use fdcanusb::FdCanUSB;
//!
//! let mut fdcanusb = FdCanUSB::open("/dev/fdcanusb", serial2::KeepSettings).expect("Failed to open serial port");
//! ```

mod bus;
mod frames;

pub use bus::FdCanUSB;
pub use frames::{CanFdFrame, FdCanUSBFrame};

pub use serial2;
