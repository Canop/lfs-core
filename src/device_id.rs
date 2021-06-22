use {
    crate::error::*,
    std::str::FromStr,
};

/// Id of a device, as can be found in MetadataExt.dev().
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeviceId {
    pub major: u32,
    pub minor: u32,
}

impl FromStr for DeviceId {
    type Err = Error;
    /// this code is based on `man 5 proc` and my stochastic interpretation
    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        match parts.len() {
            1 => Ok(parts[0].parse::<u64>()?.into()),
            2 => Ok(Self {
                major: parts[0].parse()?,
                minor: parts[1].parse()?,
            }),
            _ => Err(Error::UnexpectedFormat),
        }
    }
}

impl From<u64> for DeviceId {
    fn from(num: u64) -> Self {
        Self {
            major: (num >> 8) as u32,
            minor: (num & 0xFF) as u32,
        }
    }
}

impl DeviceId {
    pub fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }
}

#[test]
fn test_from_str() {
    assert_eq!(DeviceId::new(8, 16), DeviceId::from_str("8:16").unwrap());
}

#[test]
fn test_from_u64() {
    assert_eq!(DeviceId::new(8, 16), DeviceId::from(2064u64));
}
