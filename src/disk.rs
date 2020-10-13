use {super::*, std::fs};

/// a "block device"
#[derive(Debug, Clone)]
pub struct Disk {
    /// a name, like "sda", "sdb", "nvme0n1", etc.
    pub name: String,

    /// true for HDD, false for SSD, None for unknown
    pub rotational: Option<bool>,
}

impl Disk {
    pub fn new(name: String) -> Self {
        let rotational = sys::read_file(&format!("/sys/block/{}/queue/rotational", name))
            .ok()
            .and_then(|c| {
                match c.trim().as_ref() {
                    "0" => Some(false),
                    "1" => Some(true),
                    _ => None, // should not happen today
                }
            });
        Self { name, rotational }
    }
    pub fn disk_type(&self) -> &'static str {
        match self.rotational {
            Some(true) => "HDD",
            Some(false) => "SSD",
            None => "",
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
