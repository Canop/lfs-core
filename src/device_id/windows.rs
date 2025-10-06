use {
    super::{
        ParseDeviceIdError,
        ParseDeviceIdSnafu,
    },
    snafu::prelude::*,
    std::{
        fmt,
        str::FromStr,
    },
};

/// Id of a valume, can be found using GetVolumeInformationW
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceId {
    pub serial: u32,
}

impl fmt::Display for DeviceId {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "{:X}-{:X}", self.serial >> 16, self.serial & 0x0000_FFFF)
    }
}

impl FromStr for DeviceId {
    type Err = ParseDeviceIdError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some((high, low)) = string.split_once('-') {
            if let (Ok(high), Ok(low)) =
                (u32::from_str_radix(high, 16), u32::from_str_radix(low, 16))
            {
                let serial = (high << 16) | low;
                return Ok(Self { serial });
            }
        }

        u32::from_str_radix(string, 16)
            .ok()
            .map(|serial| Self { serial })
            .with_context(|| ParseDeviceIdSnafu { string })
    }
}

impl From<u64> for DeviceId {
    fn from(num: u64) -> Self {
        Self { serial: num as u32 }
    }
}

impl From<u32> for DeviceId {
    fn from(num: u32) -> Self {
        Self { serial: num }
    }
}

impl DeviceId {
    pub fn new(serial: u32) -> Self {
        Self { serial }
    }
}

#[test]
fn test_from_str() {
    assert_eq!(
        DeviceId::new(0xABCD_1234),
        DeviceId::from_str("ABCD-1234").unwrap()
    );
    assert_eq!(
        DeviceId::new(0xABCD_1234),
        DeviceId::from_str("ABCD1234").unwrap()
    );
}

#[test]
fn test_from_u64() {
    assert_eq!(
        DeviceId::new(0xFFFF_FFFF),
        DeviceId::from(0xFFFF_FFFF_FFFFu64)
    );
}
