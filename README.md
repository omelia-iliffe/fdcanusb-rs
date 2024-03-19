# [fdcanusb-rs](https://github.com/omelia-iliffe/fdcanusb-rs)

### For interfacing with the [fdcanusb](https://mjbots.com/products/fdcanusb) from [MJBots](https://mjbots.com/)

This crate is a work in progress but most features are implemented.  
I am very open to receiving feedback! This is the first crate I have published.

### TODO:

- [x] Implement basic functionality
- [ ] Restructure internals to use less allocations
- [ ] Implement support for the filter_id flag
- [ ] Move extra features to feature flags
- [ ] Add support for `classic_id` and `extended_id`. Currently `arbitration_id`'s are `u16`s
- [ ] Add more documentation
