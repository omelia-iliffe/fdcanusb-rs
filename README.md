# [fdcanusb-rs](https://github.com/omelia-iliffe/fdcanusb-rs)

### For interfacing with the [fdcanusb](https://mjbots.com/products/fdcanusb) from [MJBots](https://mjbots.com/)

This crate is a work in progress but most features are implemented.  
I am eager to receive feedback! This is the first crate I have published.

### Features

- `default = ["serial2"]`
- `serial2`  
  Enables re-exporting of the serial2 crate and the `FdCanUsb::open` fn.

### TODO:

- [x] Implement basic functionality
- [ ] Restructure internals to use less allocations
- [ ] Implement support for the filter_id flag
- [x] Move serial2 re-export to a feature
- [ ] Move log to a feature
- [ ] Add support for `classic_id` and `extended_id`. Currently `arbitration_id`'s are `u16`s
- [ ] Add more documentation
