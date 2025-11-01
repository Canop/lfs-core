use {
    super::{
        ParseDeviceIdError,
        ParseDeviceIdSnafu,
    },
    crate::Error,
    snafu::prelude::*,
    std::{
        fmt,
        fs,
        os::unix::fs::MetadataExt,
        path::Path,
        str::FromStr,
    },
};

/// Id of a device, as can be found in `MetadataExt.dev()`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceId {
    pub major: u32,
    pub minor: u32,
}

impl fmt::Display for DeviceId {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "{}:{}", self.major, self.minor)
    }
}

impl FromStr for DeviceId {
    type Err = ParseDeviceIdError;
    /// this code is based on `man 5 proc` and my stochastic interpretation
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        (|| {
            let mut parts = string.split(':').fuse();
            match (parts.next(), parts.next(), parts.next()) {
                (Some(major), Some(minor), None) => {
                    let major = major.parse().ok()?;
                    let minor = minor.parse().ok()?;
                    Some(Self { major, minor })
                }
                (Some(int), None, None) => {
                    let int: u64 = int.parse().ok()?;
                    Some(int.into())
                }
                _ => None,
            }
        })()
        .with_context(|| ParseDeviceIdSnafu { string })
    }
}

impl From<u64> for DeviceId {
    fn from(num: u64) -> Self {
        // need to use libc, bit format is platform-dependent
        let dev = num as libc::dev_t;
        Self {
            major: libc::major(dev) as u32,
            minor: libc::minor(dev) as u32,
        }
    }
}

impl DeviceId {
    pub fn new(
        major: u32,
        minor: u32,
    ) -> Self {
        Self { major, minor }
    }
    pub fn of_path(path: &Path) -> Result<Self, Error> {
        let md = fs::metadata(path).map_err(|e| Error::CantReadFileMetadata {
            source: e,
            path: path.to_path_buf(),
        })?;
        let dev_num = md.dev();
        Ok(Self::from(dev_num))
    }
}

#[test]
fn test_from_str() {
    assert_eq!(DeviceId::new(8, 16), DeviceId::from_str("8:16").unwrap());
}
