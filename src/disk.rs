/// what we have most looking like a physical device
#[derive(Debug, Clone)]
pub struct Disk {
    /// a name, like "sda", "sdc", "nvme0n1", etc.
    #[cfg(not(windows))]
    pub name: String,

    /// true for HDD, false for SSD, None for unknown.
    /// This information isn't reliable for USB devices
    pub rotational: Option<bool>,

    /// whether the system thinks the media is removable.
    /// Seems reliable when not mapped
    pub removable: Option<bool>,

    /// whether the disk is read-only
    pub read_only: Option<bool>,

    /// whether it's a RAM disk
    pub ram: bool,

    /// disk image (Mac only right now)
    pub image: bool,

    /// whether it's on LVM
    pub lvm: bool,

    /// whether it's a crypted disk
    pub crypted: bool,

    /// whether it's a remote disk
    #[cfg(windows)]
    pub remote: bool,
}

impl Disk {
    /// a synthetic code trying to express the essence of the type of media,
    /// an empty str being returned when information couldn't be gathered.
    /// This code is for humans and may change in future minor versions.
    pub fn disk_type(&self) -> &'static str {
        if self.ram {
            "RAM"
        } else if self.image {
            "imag"
        } else if self.crypted {
            "crypt"
        } else if self.lvm {
            "LVM"
        } else {
            match (self.removable, self.rotational) {
                (Some(true), _) => "remov",
                (Some(false), Some(true)) => "HDD",
                (Some(false), Some(false)) => "SSD",
                _ => "",
            }
        }
    }
}
