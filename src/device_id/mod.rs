use snafu::prelude::*;

#[cfg(target_family = "unix")]
mod unix;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_family = "unix")]
pub use unix::DeviceId;
#[cfg(target_os = "windows")]
pub use windows::DeviceId;

#[derive(Debug, Snafu)]
#[snafu(display("Could not parse {string} as a device id"))]
pub struct ParseDeviceIdError {
    string: String,
}
