use std::collections::HashMap;

use listdisk_rs::win32::storagepool::{
    StoragePool,
    StoragePoolToVolume,
};

use {
    listdisk_rs::win32::volume_wmi::Volume,
    wmi::WMIError,
};

use {
    windows::Win32::{
        Foundation::{
            CloseHandle,
            ERROR_MORE_DATA,
        },
        Storage::FileSystem::{
            CreateFileW,
            FILE_SHARE_READ,
            FILE_SHARE_WRITE,
            IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS,
            OPEN_EXISTING,
        },
        System::{
            IO::DeviceIoControl,
            Ioctl::{
                DISK_EXTENT,
                VOLUME_DISK_EXTENTS,
            },
        },
    },
    wmi::{
        FilterValue,
        WMIConnection,
        WMIResult,
    },
};

use crate::windows::volume::{
    VolumeKind,
    VolumeName,
};

pub fn volume_kind_detect(verbatim_path: &VolumeName) -> VolumeKind {
    let handle = match unsafe {
        CreateFileW(
            verbatim_path.as_pcwstr_no_trailing_backslash(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            Default::default(),
            None,
        )
    } {
        Ok(handle) => handle,
        Err(_) => return VolumeKind::Unknown,
    };

    let mut extents_buffer = VOLUME_DISK_EXTENTS {
        NumberOfDiskExtents: 0,
        Extents: [DISK_EXTENT {
            DiskNumber: 0,
            StartingOffset: 0,
            ExtentLength: 0,
        }],
    };

    let mut bytes_returned: u32 = 0;

    let result = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS,
            None,
            0,
            Some(&mut extents_buffer as *mut _ as *mut _),
            std::mem::size_of::<VOLUME_DISK_EXTENTS>() as u32,
            Some(&mut bytes_returned),
            None,
        )
    };

    let _ = unsafe { CloseHandle(handle) };

    match result {
        Ok(_) => match extents_buffer.NumberOfDiskExtents {
            1 if is_volume_storage_space(verbatim_path).is_ok_and(|x| x) => {
                VolumeKind::StorageSpace
            }
            1 => VolumeKind::Simple {
                disk_number: extents_buffer.Extents[0].DiskNumber,
            },
            _ => VolumeKind::DynamicDisk,
        },
        Err(error) if error.code() == ERROR_MORE_DATA.to_hresult() => VolumeKind::DynamicDisk,
        Err(_) => VolumeKind::Unknown,
    }
}

fn is_volume_storage_space(verbatim_path: &VolumeName) -> WMIResult<bool> {
    let wmi = WMIConnection::with_namespace_path(r#"ROOT\Microsoft\Windows\Storage"#)?;
    let map = HashMap::from([(
        "Path".into(),
        FilterValue::String(verbatim_path.to_string()),
    )]);

    let volume = wmi
        .filtered_query::<Volume>(&map)?
        .pop()
        .ok_or(WMIError::ResultEmpty)?;

    let storage_pool = wmi.associators::<StoragePool, StoragePoolToVolume>(&volume.obj_path)?;
    Ok(!storage_pool.is_empty())
}
