use {
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
    /// number of inodes
    pub files: u64,
    /// number of free inodes
    pub ffree: u64,
    /// number of free inodes for underpriviledged users
    pub favail: u64,
}

impl Stats {
    pub fn from(mount_point: &Path) -> Option<Self> {
        let c_mount_point = CString::new(mount_point.as_os_str().as_bytes()).unwrap();
        unsafe {
            let mut statvfs = mem::MaybeUninit::<libc::statvfs>::uninit();
            let code = libc::statvfs(c_mount_point.as_ptr(), statvfs.as_mut_ptr());
            match code {
                0 => {
                    // good
                    let statvfs = statvfs.assume_init();
                    Some(Stats {
                        bsize: statvfs.f_bsize as u64,
                        blocks: statvfs.f_blocks as u64,
                        bfree: statvfs.f_bfree as u64,
                        bavail: statvfs.f_bavail as u64,
                        files: statvfs.f_files as u64,
                        ffree: statvfs.f_ffree as u64,
                        favail: statvfs.f_favail as u64,
                    })
                }
                _ => {
                    // the filesystem wasn't found, it's a strange one, for example a
                    // docker one
                    None
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
    pub fn inodes_used(&self) -> u64 {
        self.files - self.favail // this will panic on unconsistent data
    }
    pub fn inodes_use_share(&self) -> f64 {
        if self.files == 0 {
            0.0
        } else {
            self.inodes_used() as f64 / self.files as f64
        }
    }
    pub fn use_share(&self) -> f64 {
        if self.size() == 0 {
            0.0
        } else {
            self.used() as f64 / (self.size() as f64)
        }
    }
}
