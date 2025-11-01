use {
    super::{
        ParseDeviceIdError,
        ParseDeviceIdSnafu,
    },
    crate::WindowsApiSnafu,
    snafu::prelude::*,
    std::{
        ffi::OsStr,
        fmt,
        os::windows::ffi::OsStrExt,
        path::Path,
        str::FromStr,
    },
    windows::{
        Win32::Storage::FileSystem::{
            GetVolumeInformationW,
            GetVolumeNameForVolumeMountPointW,
            GetVolumePathNameW,
        },
        core::PCWSTR,
    },
};
/// Id of a volume, can be found using GetVolumeInformationW
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
    /// Determine the DeviceId for a given file path
    pub fn of_path(path: &Path) -> Result<Self, crate::Error> {
        unsafe {
            let path_wide: Vec<u16> = OsStr::new(path)
                .encode_wide()
                .chain(std::iter::once(0)) // null terminator
                .collect();

            // Step 1: Get the volume path from the file path
            let mut volume_path_buf = vec![0u16; 260]; // MAX_PATH
            GetVolumePathNameW(PCWSTR(path_wide.as_ptr()), &mut volume_path_buf).context(
                WindowsApiSnafu {
                    api: "GetVolumePathNameW",
                },
            )?;

            // Step 2: Get the volume GUID from the volume path
            let mut volume_guid_buf = vec![0u16; 260]; // MAX_PATH
            GetVolumeNameForVolumeMountPointW(
                PCWSTR(volume_path_buf.as_ptr()),
                &mut volume_guid_buf,
            )
            .context(WindowsApiSnafu {
                api: "GetVolumeNameForVolumeMountPointW",
            })?;

            // Step 3: Get serial number from the GUID
            let mut serial: u32 = 0;
            GetVolumeInformationW(
                PCWSTR(volume_guid_buf.as_ptr()),
                None,
                Some(&mut serial),
                None,
                None,
                None,
            )
            .context(WindowsApiSnafu {
                api: "GetVolumeInformationW",
            })?;

            Ok(Self { serial })
        }
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
