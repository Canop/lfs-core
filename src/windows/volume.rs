use {
    crate::{
        DeviceId,
        Mount,
        MountInfo,
        WindowsApiSnafu,
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
    },
    windows::{
        Win32::{
            Foundation::{
                ERROR_MORE_DATA,
                HANDLE,
                MAX_PATH,
            },
            Storage::FileSystem::{
                FindFirstVolumeW,
                FindNextVolumeW,
                FindVolumeClose,
                GetVolumeInformationW,
                GetVolumePathNamesForVolumeNameW,
            },
            System::SystemServices::FILE_READ_ONLY_VOLUME,
        },
        core::PCWSTR,
    },
};

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
        let end = self.full_path.len().saturating_sub(1);
        let s = String::from_utf16_lossy(&self.full_path[..end]);
        f.debug_tuple("VolumeName").field(&s).finish()
    }
}


impl fmt::Display for VolumeName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

    pub fn to_dysk_mounts(&self) -> Result<Vec<Mount>, crate::Error> {
        let mounts = self.mount_points()?;

        if mounts.is_empty() {
            return Ok(Vec::new());
        }

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
                    fs: self.name.to_string(),
                    fs_type: file_system_name.clone(),
                    bound: false,
                };

                Mount {
                    info,
                    fs_label: Some(label.clone()),
                    disk: None,
                    stats: Err(crate::StatsError::Excluded),
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
