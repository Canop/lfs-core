use crate::Inodes;

/// inode & blocs information
///
/// The semantics is mostly the one of statvfs, with addition of
///  bused which is necesssary for volumes freely growing in containers
#[derive(Debug, Clone)]
pub struct Stats {
    /// block size
    pub bsize: u64,
    /// number of blocks
    pub blocks: u64,
    /// not provided by statvfs
    pub bused: u64,
    /// number of free blocks
    pub bfree: u64,
    /// number of free blocks for underprivileged users
    pub bavail: u64,
    /// information relative to inodes, if available
    pub inodes: Option<Inodes>,
}

#[derive(Debug, snafu::Snafu, Clone, Copy, PartialEq, Eq)]
#[snafu(visibility(pub(crate)))]
pub enum StatsError {
    #[snafu(display("Could not stat mount point"))]
    Unreachable,

    #[snafu(display("Unconsistent stats"))]
    Unconsistent,

    /// Options made us not even try
    #[snafu(display("Excluded"))]
    Excluded,
}

impl Stats {
    pub fn size(&self) -> u64 {
        self.bsize * self.blocks
    }
    pub fn available(&self) -> u64 {
        self.bsize * self.bavail
    }
    /// Space used in the volume (including unreadable fs metadata)
    pub fn used(&self) -> u64 {
        self.bsize * self.bused
    }
    pub fn use_share(&self) -> f64 {
        if self.blocks == 0 {
            0.0
        } else {
            (self.blocks - self.bfree) as f64 / self.blocks as f64
        }
    }
}
