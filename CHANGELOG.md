# Version 0.3.0 - 09-08-2024
- **Major**: Changed `CanFdFrame::new` and `CanFdFrame::new_with_data` to `CanFdFrame::new_standard` and `CanFdFrame::new_with_flags` to return Err if more than 64 bytes are provided.
- **Major**: Added `error.rs` and error types to reduce usage of `std::io::Error`.
- **Major**: Fixed an incorrect `/ 2` when calculating the `padding_len` for frames.
- **Minor**: Moved the `flush` method to the `serial2` feature to use the `discard_buffers` method in addition to `flush`.
- **Minor**: Increased the `read_ok` buffer to 100 bytes to read the error message if one occurs. Ideally this would read all bytes not just 100.
# Version 0.2.0 - 25-03-2024
- **Major**: Changed License to apache-2.0
- **Minor**: Added default feature `serial2`, and moved `serial2` specific code to `serial2` feature flag.
# Version 0.1.0 - 19-03-2024
- Initial release
