use {
    crate::{
        DeviceId,
        Disk,
        Mount,
        MountInfo,
        ReadOptions,
        Stats,
        StatsError,
        WindowsApiSnafu,
        windows::volume::volume_utils::volume_kind_detect,
    },
    snafu::{
        ResultExt,
        prelude::*,
    },
    std::{
        ffi::OsString,
        fmt,
        os::windows::ffi::OsStringExt,
        path::PathBuf,
        ptr,
    },
    windows::{
        Win32::{
            Foundation::{
                CloseHandle,
                ERROR_MORE_DATA,
                HANDLE,
                MAX_PATH,
            },
            Storage::FileSystem::{
                CreateFileW,
                FILE_SHARE_READ,
                FILE_SHARE_WRITE,
                FindFirstVolumeW,
                FindNextVolumeW,
                FindVolumeClose,
                GetDiskFreeSpaceExW,
                GetDriveTypeW,
                GetVolumeInformationW,
                GetVolumePathNamesForVolumeNameW,
                OPEN_EXISTING,
            },
            System::{
                IO::DeviceIoControl,
                Ioctl::{
                    DEVICE_SEEK_PENALTY_DESCRIPTOR,
                    IOCTL_STORAGE_QUERY_PROPERTY,
                    PropertyStandardQuery,
                    STORAGE_PROPERTY_QUERY,
                    StorageDeviceSeekPenaltyProperty,
                },
                SystemServices::FILE_READ_ONLY_VOLUME,
                WindowsProgramming::{
                    DRIVE_CDROM,
                    DRIVE_FIXED,
                    DRIVE_RAMDISK,
                    DRIVE_REMOTE,
                    DRIVE_REMOVABLE,
                },
            },
        },
        core::PCWSTR,
    },
};

mod volume_utils;

#[derive(Debug, Clone)]
pub enum VolumeKind {
    Simple { disk_number: u32 },
    DynamicDisk,
    StorageSpace,
    Unknown,
}

trait WideStringExt {
    fn wcslen(&self) -> usize;
}

impl WideStringExt for [u16] {
    fn wcslen(&self) -> usize {
        self.iter().position(|&c| c == 0).unwrap_or(self.len())
    }
}

#[derive(Debug, Snafu)]
#[snafu(display("Invalid volume name: {:?}", volume_name))]
pub struct VolumeNameError {
    volume_name: OsString,
}

pub struct VolumeName {
    full_path: Vec<u16>,
    device_path: Vec<u16>,
}

impl fmt::Debug for VolumeName {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let s = String::from_utf16_lossy(&self.full_path[..self.full_path.wcslen()]);
        f.debug_tuple("VolumeName").field(&s).finish()
    }
}

impl fmt::Display for VolumeName {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        let s = String::from_utf16_lossy(&self.full_path[..self.full_path.wcslen()]);
        write!(f, "{}", s)
    }
}

impl VolumeName {
    const PREFIX: &[u16] = &[
        b'\\' as u16,
        b'\\' as u16,
        b'?' as u16,
        b'\\' as u16,
        b'V' as u16,
        b'o' as u16,
        b'l' as u16,
        b'u' as u16,
        b'm' as u16,
        b'e' as u16,
        b'{' as u16,
    ];
    const SUFFIX: &[u16] = &[b'}' as u16, b'\\' as u16, 0];

    pub fn from_null_terminated(buffer: &[u16]) -> Result<Self, VolumeNameError> {
        let length = buffer.wcslen();

        let full_path = &buffer[..=length];

        if !buffer.starts_with(Self::PREFIX) || !buffer[..=length].ends_with(Self::SUFFIX) {
            return Err(VolumeNameError {
                volume_name: OsString::from_wide(full_path),
            });
        }

        let mut device_path = full_path[..full_path.len() - 1].to_vec();
        if let Some(last) = device_path.last_mut() {
            *last = 0;
        }

        Ok(VolumeName {
            full_path: full_path.to_vec(),
            device_path,
        })
    }

    pub fn to_uuid(&self) -> Option<String> {
        let uuid = &self.full_path[Self::PREFIX.len()..self.full_path.len() - Self::SUFFIX.len()];

        String::from_utf16(uuid).ok()
    }

    pub fn as_pcwstr(&self) -> PCWSTR {
        PCWSTR(self.full_path.as_ptr())
    }

    pub fn as_pcwstr_no_trailing_backslash(&self) -> PCWSTR {
        PCWSTR(self.device_path.as_ptr())
    }
}

#[derive(Debug)]
struct VolumeInformation {
    label: String,
    serial_number: u32,
    read_only: bool,
    file_system_name: String,
}

#[derive(Debug)]
pub struct Volume {
    name: VolumeName,
}

impl Volume {
    pub fn new(name: VolumeName) -> Self {
        Self { name }
    }

    pub fn to_dysk_mounts(
        &self,
        options: &ReadOptions,
    ) -> Result<Vec<Mount>, crate::Error> {
        let mounts = self.mount_points()?;

        if mounts.is_empty() {
            return Ok(Vec::new());
        }

        let disk = self.disk_info();

        let stats = if !options.remote_stats && disk.as_ref().is_some_and(|disk| disk.remote) {
            Err(StatsError::Excluded)
        } else {
            self.volume_stats()
        };

        let VolumeInformation {
            serial_number,
            label,
            file_system_name,
            ..
        } = self.volume_information()?;

        Ok(mounts
            .into_iter()
            .map(|mount_point| {
                let info = MountInfo {
                    id: None,
                    parent: None,
                    dev: DeviceId::from(serial_number),
                    root: mount_point.clone(),
                    mount_point,
                    options: Vec::new(),
                    fs: self.name.to_string(),
                    fs_type: file_system_name.clone(),
                    bound: false,
                };

                Mount {
                    info,
                    fs_label: Some(label.clone()),
                    disk: disk.clone(),
                    stats: stats.clone(),
                    uuid: self.name.to_uuid(),
                    part_uuid: None,
                }
            })
            .collect())
    }

    fn mount_points(&self) -> Result<Vec<PathBuf>, crate::Error> {
        let mut char_count = MAX_PATH + 1;

        loop {
            let mut mounts = vec![0u16; char_count as usize];

            match unsafe {
                GetVolumePathNamesForVolumeNameW(
                    self.name.as_pcwstr(),
                    Some(&mut mounts),
                    &mut char_count,
                )
            } {
                Ok(_) => {
                    return Ok(mounts[..char_count as usize]
                        .split(|&c| c == 0)
                        .filter(|s| !s.is_empty())
                        .map(OsString::from_wide)
                        .map(PathBuf::from)
                        .collect());
                }
                Err(error) if error.code() == ERROR_MORE_DATA.to_hresult() => continue,
                Err(error) => {
                    return Err(error).context(WindowsApiSnafu {
                        api: "GetVolumePathNamesForVolumeNameW",
                    });
                }
            }
        }
    }

    fn volume_information(&self) -> Result<VolumeInformation, crate::Error> {
        // The max supported buffer size for GetVolumeInformationW
        const BUFFER_SIZE: usize = (MAX_PATH + 1) as usize;

        let mut serial_number: u32 = 0;
        let mut flags: u32 = 0;

        let mut volume_label_buffer: [u16; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut file_system_name_buffer: [u16; BUFFER_SIZE] = [0; BUFFER_SIZE];

        unsafe {
            GetVolumeInformationW(
                self.name.as_pcwstr(),
                Some(&mut volume_label_buffer),
                Some(&mut serial_number),
                None,
                Some(&mut flags),
                Some(&mut file_system_name_buffer),
            )
            .context(WindowsApiSnafu {
                api: "GetVolumeInformationW",
            })?;
        }

        Ok(VolumeInformation {
            label: String::from_utf16_lossy(&volume_label_buffer[..volume_label_buffer.wcslen()]),
            serial_number,
            read_only: flags & FILE_READ_ONLY_VOLUME != 0,
            file_system_name: String::from_utf16_lossy(
                &file_system_name_buffer[..file_system_name_buffer.wcslen()],
            ),
        })
    }

    fn disk_info(&self) -> Option<Disk> {
        let volume_kind = self.volume_kind();

        let drive_type = unsafe { GetDriveTypeW(self.name.as_pcwstr()) };

        let (removable, remote, ram) = match drive_type {
            DRIVE_REMOVABLE => (Some(true), false, false),
            DRIVE_FIXED => (Some(false), false, false),
            DRIVE_REMOTE => (Some(false), true, false),
            DRIVE_CDROM => (Some(true), false, false),
            DRIVE_RAMDISK => (Some(false), false, true),
            _ => return None,
        };

        let rotational = if let VolumeKind::Simple { disk_number } = volume_kind {
            is_disk_rotational(disk_number)
        } else {
            None
        };

        Some(Disk {
            rotational,
            removable,
            read_only: self.volume_information().map(|info| info.read_only).ok(),
            ram,
            image: false,
            lvm: matches!(volume_kind, VolumeKind::DynamicDisk)
                || matches!(volume_kind, VolumeKind::StorageSpace),
            crypted: false,
            remote,
        })
    }

    fn volume_kind(&self) -> VolumeKind {
        let kind = volume_kind_detect(&self.name);

        // only pollute stderr in debug builds
        #[cfg(debug_assertions)]
        dbg!(&kind);

        kind
    }

    fn volume_stats(&self) -> Result<Stats, StatsError> {
        let mut free_bytes_available: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut total_free_bytes: u64 = 0;

        unsafe {
            GetDiskFreeSpaceExW(
                self.name.as_pcwstr(),
                Some(ptr::addr_of_mut!(free_bytes_available).cast()),
                Some(ptr::addr_of_mut!(total_bytes).cast()),
                Some(ptr::addr_of_mut!(total_free_bytes).cast()),
            )
            .map_err(|_| StatsError::Unreachable)?;
        }

        Ok(Stats {
            size: total_bytes,
            free: free_bytes_available,
            inodes: None,
        })
    }
}

pub fn get_volumes() -> Result<Vec<Volume>, crate::Error> {
    let mut volume_names = Vec::new();

    let mut volume_name_buffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    let handle: HANDLE = unsafe {
        FindFirstVolumeW(&mut volume_name_buffer).context(WindowsApiSnafu {
            api: "FindFirstVolumeW",
        })?
    };

    loop {
        if let Ok(name) = VolumeName::from_null_terminated(&volume_name_buffer) {
            volume_names.push(name);
        };

        if unsafe { FindNextVolumeW(handle, &mut volume_name_buffer).is_err() } {
            // Break not return so that the handle can be closed
            break;
        }
    }

    unsafe {
        FindVolumeClose(handle).context(WindowsApiSnafu {
            api: "FindVolumeClose",
        })?
    };

    Ok(volume_names.into_iter().map(Volume::new).collect())
}

fn is_disk_rotational(disk_number: u32) -> Option<bool> {
    let path: Vec<u16> = format!("\\\\.\\PhysicalDrive{}\0", disk_number)
        .encode_utf16()
        .collect();

    let handle = match unsafe {
        CreateFileW(
            PCWSTR(path.as_ptr()),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            Default::default(),
            None,
        )
    } {
        Ok(handle) => handle,
        Err(_) => return None,
    };

    let query = STORAGE_PROPERTY_QUERY {
        PropertyId: StorageDeviceSeekPenaltyProperty,
        QueryType: PropertyStandardQuery,
        AdditionalParameters: [0; 1],
    };

    let mut seek_penalty = DEVICE_SEEK_PENALTY_DESCRIPTOR {
        Version: 0,
        Size: 0,
        IncursSeekPenalty: false,
    };

    let mut bytes_returned = 0u32;

    let result = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_STORAGE_QUERY_PROPERTY,
            Some(ptr::addr_of!(query).cast()),
            std::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
            Some(ptr::addr_of_mut!(seek_penalty).cast()),
            std::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32,
            Some(&mut bytes_returned),
            None,
        )
    };

    let _ = unsafe { CloseHandle(handle) };

    match result {
        Ok(_) => Some(seek_penalty.IncursSeekPenalty),
        _ => None,
    }
}
