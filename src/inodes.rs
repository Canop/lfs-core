
/// inode information
///
/// This structure isn't built if data aren't consistent
#[derive(Debug, Clone)]
pub struct Inodes {
    /// number of inodes
    pub files: u64,
    /// number of free inodes
    pub ffree: u64,
    /// number of free inodes for underpriviledged users
    pub favail: u64,
}

impl Inodes {
    pub fn new(files: u64, ffree: u64, favail: u64) -> Option<Self> {
        if files > 0 && ffree <= files && favail <= files {
            Some(Self { files, ffree, favail })
        } else {
            None
        }
    }
    pub fn used(&self) -> u64 {
        self.files - self.favail
    }
    pub fn use_share(&self) -> f64 {
        self.used() as f64 / self.files as f64
    }
}
