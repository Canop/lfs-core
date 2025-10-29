use snafu::prelude::*;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::DeviceId;
#[cfg(windows)]
pub use windows::DeviceId;

#[derive(Debug, Snafu)]
#[snafu(display("Could not parse {string} as a device id"))]
pub struct ParseDeviceIdError {
    string: String,
}
