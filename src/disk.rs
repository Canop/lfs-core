use {
    super::*,
    lazy_regex::*,
    std::fs,
};

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

    /// whether it's a RAM disk
    pub ram: bool,
}

impl Disk {
    pub fn new(name: String) -> Self {
        let rotational = sys::read_file_as_bool(&format!("/sys/block/{}/queue/rotational", name));
        let removable = sys::read_file_as_bool(&format!("/sys/block/{}/removable", name));
        let ram = regex_is_match!(r#"^zram\d*$"#, &name);
        Self { name, rotational, removable , ram }
    }
    /// a synthetic code trying to express the essence of the type of media,
    /// an empty str being returned when information couldn't be gathered.
    /// This code is for humans and may change in future minor versions.
    pub fn disk_type(&self) -> &'static str {
        match (self.removable, self.rotational, self.ram) {
            (_, _, true) => "RAM",
            (Some(true), _, _) => "rem",
            (Some(false), Some(true), _) => "HDD",
            (Some(false), Some(false), _) => "SSD",
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
