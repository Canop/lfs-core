use crate::Inodes;

/// inode & storage usage information
#[derive(Debug, Clone)]
pub struct Stats {
    /// number of bytes
    pub size: u64,
    /// number of free bytes
    pub free: u64,
    /// information relative to inodes, if available
    pub inodes: Option<Inodes>,
}

impl Stats {
    pub fn size(&self) -> u64 {
        self.size
    }
    pub fn available(&self) -> u64 {
        self.free
    }
    /// Space used in the volume (including unreadable fs metadata)
    pub fn used(&self) -> u64 {
        self.size - self.free
    }
    pub fn use_share(&self) -> f64 {
        if self.free == 0 {
            0.0
        } else {
            (self.size - self.free) as f64 / self.size as f64
        }
    }
}
