use {
    super::*,
    lazy_regex::*,
};

/// what we have most looking like a physical device
#[derive(Debug, Clone)]
pub struct Disk {
    /// a name, like "sda", "sdc", "nvme0n1", etc.
    pub name: String,

    /// true for HDD, false for SSD, None for unknown.
    /// This information isn't reliable for USB devices
    pub rotational: Option<bool>,

    /// whether the system thinks the media is removable.
    /// Seems reliable when not mapped
    pub removable: Option<bool>,

    /// whether it's a RAM disk
    pub ram: bool,

    /// whether it's on LVM
    pub lvm: bool,

    /// whether it's a crypted disk
    pub crypted: bool,
}

impl Disk {
    pub fn new(name: String) -> Self {
        let rotational = sys::read_file_as_bool(&format!("/sys/block/{}/queue/rotational", name));
        let removable = sys::read_file_as_bool(&format!("/sys/block/{}/removable", name));
        let ram = regex_is_match!(r#"^zram\d*$"#, &name);
        let dm_uuid = sys::read_file(&format!("/sys/block/{}/dm/uuid", name)).ok();
        let crypted = dm_uuid.as_ref().map_or(false, |uuid| uuid.starts_with("CRYPT-"));
        let lvm = dm_uuid.map_or(false, |uuid| uuid.starts_with("LVM-"));
        Self { name, rotational, removable , ram, lvm, crypted }
    }
    /// a synthetic code trying to express the essence of the type of media,
    /// an empty str being returned when information couldn't be gathered.
    /// This code is for humans and may change in future minor versions.
    pub fn disk_type(&self) -> &'static str {
        if self.ram {
            "RAM"
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

