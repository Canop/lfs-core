mod volume;

use ::snafu::prelude::*;

use std::{
    os::windows::ffi::OsStrExt,
    path::Path,
};

use windows::{
    Win32::Storage::FileSystem::GetVolumeInformationW,
    core::PCWSTR,
};

use crate::{
    Error,
    Mount,
    ReadOptions,
    WindowsApiSnafu,
    windows::volume::get_volumes,
};

/// Read all the mount points and load basic information on them
pub fn read_mounts(options: &ReadOptions) -> Result<Vec<Mount>, Error> {
    Ok(get_volumes()?
        .into_iter()
        .flat_map(|volume| volume.to_dysk_mounts(options).ok())
        .flatten()
        .collect())
}

/// Get a volume serial number for a provided path
pub fn volume_serial_for_path(path: impl AsRef<Path>) -> Result<u32, crate::Error> {
    let path_wide: Vec<u16> = path
        .as_ref()
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut serial_number: u32 = 0;

    unsafe {
        GetVolumeInformationW(
            PCWSTR(path_wide.as_ptr()),
            None,
            Some(&mut serial_number),
            None,
            None,
            None,
        )
        .context(WindowsApiSnafu {
            api: "GetVolumeInformationW",
        })?;
    }

    Ok(serial_number)
}
