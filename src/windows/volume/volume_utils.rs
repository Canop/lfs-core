use std::{
    os::windows::io::{
        FromRawHandle,
        OwnedHandle,
    },
    ptr,
};

use windows::Win32::{
    Foundation::{
        ERROR_MORE_DATA,
        HANDLE,
    },
    Storage::FileSystem::{
        BusTypeSpaces,
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
            IOCTL_STORAGE_QUERY_PROPERTY,
            PropertyStandardQuery,
            STORAGE_DEVICE_DESCRIPTOR,
            STORAGE_PROPERTY_QUERY,
            StorageDeviceProperty,
            VOLUME_DISK_EXTENTS,
        },
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
    let _handle = unsafe { OwnedHandle::from_raw_handle(handle.0) };

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

    match result {
        Ok(_) => match extents_buffer.NumberOfDiskExtents {
            1 if is_volume_storage_space(handle) => VolumeKind::StorageSpace,
            1 => VolumeKind::Simple {
                disk_number: extents_buffer.Extents[0].DiskNumber,
            },
            _ => VolumeKind::DynamicDisk,
        },
        Err(error) if error.code() == ERROR_MORE_DATA.to_hresult() => VolumeKind::DynamicDisk,
        Err(_) => VolumeKind::Unknown,
    }
}

fn is_volume_storage_space(handle: HANDLE) -> bool {
    let query = STORAGE_PROPERTY_QUERY {
        PropertyId: StorageDeviceProperty,
        QueryType: PropertyStandardQuery,
        AdditionalParameters: [0; 1],
    };

    let mut descriptor = STORAGE_DEVICE_DESCRIPTOR::default();

    let result = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_STORAGE_QUERY_PROPERTY,
            Some(ptr::addr_of!(query).cast()),
            std::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
            Some(ptr::addr_of_mut!(descriptor).cast()),
            std::mem::size_of::<STORAGE_DEVICE_DESCRIPTOR>() as u32,
            None,
            None,
        )
    };

    result.is_ok_and(|_| descriptor.BusType == BusTypeSpaces)
}
