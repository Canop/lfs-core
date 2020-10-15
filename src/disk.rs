use {super::*, std::fs};

/// a "block device"
#[derive(Debug, Clone)]
pub struct Disk {
    /// a name, like "sda", "sdc", "nvme0n1", etc.
    pub name: String,

    /// true for HDD, false for SSD, None for unknown.
    /// This information isn't reliable for USB devices
    pub rotational: Option<bool>,

    /// whether the system thinks the media is removable.
    /// Seems reliable.
    pub removable: Option<bool>,
}

impl Disk {
    pub fn new(name: String) -> Self {
        let rotational = sys::read_file_as_bool(&format!("/sys/block/{}/queue/rotational", name));
        let removable = sys::read_file_as_bool(&format!("/sys/block/{}/removable", name));
        Self { name, rotational, removable }
    }
    /// a synthetic code trying to express the essence of the type of media,
    /// an empty str being returned when information couldn't be gathered.
    /// This code is for humans and may change in future minor versions.
    pub fn disk_type(&self) -> &'static str {
        match (self.removable, self.rotational) {
            (Some(true), _) => "rem",
            (Some(false), Some(true)) => "HDD",
            (Some(false), Some(false)) => "SSD",
            _ => "",
        }
    }
}

pub fn read_disks() -> Result<Vec<Disk>> {
    Ok(fs::read_dir("/sys/block")?
        .flatten()
        .map(|e| e.file_name().into_string().unwrap())
        .map(Disk::new)
        .collect())
}
