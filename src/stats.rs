use {
    crate::Inodes,
    std::{
        ffi::CString,
        mem,
        os::unix::ffi::OsStrExt,
        path::Path,
    },
};

/// inode & blocs information given by statvfs
#[derive(Debug, Clone)]
pub struct Stats {
    /// block size
    pub bsize: u64,
    /// number of blocks
    pub blocks: u64,
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
                    let bsize = statvfs.f_bsize as u64;
                    let blocks = statvfs.f_blocks as u64;
                    let bfree = statvfs.f_bfree as u64;
                    let bavail = statvfs.f_bavail as u64;
                    if bsize == 0 || blocks == 0 || bfree > blocks || bavail > blocks {
                        // unconsistent or void data
                        return Err(StatsError::Unconsistent);
                    }

                    // inodes info, will be checked in Inodes::new
                    let files = statvfs.f_files as u64;
                    let ffree = statvfs.f_ffree as u64;
                    let favail = statvfs.f_favail as u64;
                    let inodes = Inodes::new(files, ffree, favail);

                    Ok(Stats {
                        bsize,
                        blocks,
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
    pub fn used(&self) -> u64 {
        self.size() - self.available()
    }
    pub fn use_share(&self) -> f64 {
        if self.size() == 0 {
            0.0
        } else {
            self.used() as f64 / (self.size() as f64)
        }
    }
}
