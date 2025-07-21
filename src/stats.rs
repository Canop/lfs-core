use {
    crate::Inodes,
    std::{
        ffi::CString,
        mem,
        os::unix::ffi::OsStrExt,
        path::Path,
    },
};

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
    pub fn from(mount_point: &Path) -> Result<Self, StatsError> {
        let c_mount_point = CString::new(mount_point.as_os_str().as_bytes()).unwrap();
        unsafe {
            let mut statvfs = mem::MaybeUninit::<libc::statvfs>::uninit();
            let code = libc::statvfs(c_mount_point.as_ptr(), statvfs.as_mut_ptr());
            match code {
                0 => {
                    let statvfs = statvfs.assume_init();

                    // blocks info
                    let bsize = statvfs.f_bsize;
                    let blocks = statvfs.f_blocks;
                    let bfree = statvfs.f_bfree;
                    let bavail = statvfs.f_bavail;
                    if bsize == 0 || blocks == 0 || bfree > blocks || bavail > blocks {
                        // unconsistent or void data
                        return Err(StatsError::Unconsistent);
                    }

                    // statvfs doesn't provide bused
                    let bused = blocks - bavail;

                    // inodes info, will be checked in Inodes::new
                    let files = statvfs.f_files;
                    let ffree = statvfs.f_ffree;
                    let favail = statvfs.f_favail;
                    let inodes = Inodes::new(files, ffree, favail);

                    Ok(Stats {
                        bsize,
                        blocks,
                        bused,
                        bfree,
                        bavail,
                        inodes,
                    })
                }
                _ => {
                    // the filesystem wasn't found, it's a strange one, for example a
                    // docker one, or a disconnected remote one
                    Err(StatsError::Unreachable)
                }
            }
        }
    }
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
