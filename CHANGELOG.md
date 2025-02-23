# Version 0.7.1 - 28-01-2025
- **Minor**: Fixed broken windows build
# Version 0.7.0 - 15-01-2025
- **Major**: Removed generic transport in favour of serial2::SerialPort
- **Major**: Removed serial2 feature
- **Major**: Removed serial_settings arg in `FdCanUsb::open` method
- **Major**: Added sensible default serial2::Settings in `FdCanUsb::open`, fixing a sync issue on unix
- **Minor**: changed from `thiserror` to `derive_more`
# Version 0.6.3 - 21-11-2024
- **Add** Added `FdCanUsb::Write` and `FdCanUsb::Read` methods
# Version 0.6.2 - 06-09-2024
- **Minor**: Fixed flag cases being incorrect, leading to unexpected behaviour.
# Version 0.6.1 - 21-08-2024
- **Minor**: Added default feature `log` to enable logging.
- **Minor**: Reduced unnecessary info logs.
# Version 0.6.0 - 16-08-2024
- **Major**: Added a read Buffer. Default buffer size is 256, which should be enough for most cases.
- **Major**: Added `new_with_buffers` to `FdCanUsb` to allow for custom buffers.
- **Major**: Changed `read_ok` and `read_response` to use the buffer and fix an issue of missing bytes.
- **Major**: Made `read_ok`, `read_response`, and `write` private.
- **Minor**: Changed packet logs to `Debug` level.
# Version 0.5.0 - 09-08-2024
- **Major**: Fixed error types not being public.
# Version 0.4.0 - 09-08-2024
- **Major**: Added `error.rs` and error types to reduce usage of `std::io::Error`.
- **Major**: Fixed an incorrect `/ 2` when calculating the `padding_len` for frames.
# Version 0.3.0 - 17-06-2024
- **Major**: Changed `CanFdFrame::new` and `CanFdFrame::new_with_data` to return Err if more than 64 bytes are provided.
- **Minor**: Moved the `flush` method to the `serial2` feature to use the `discard_buffers` method in addition to `flush`.
- **Minor**: Increased the `read_ok` buffer to 100 bytes to read the error message if one occurs. Ideally this would read all bytes not just 100.
# Version 0.2.0 - 25-03-2024
- **Major**: Changed License to apache-2.0
- **Minor**: Added default feature `serial2`, and moved `serial2` specific code to `serial2` feature flag.
# Version 0.1.0 - 19-03-2024
- Initial release
