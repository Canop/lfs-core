use {
    super::*,
    std::{ffi::CString, mem, os::unix::ffi::OsStrExt, path::Path},
};

/// inode & blocs information given by statvfs
#[derive(Debug, Clone)]
pub struct Stats {
    pub bsize: u64,
    pub blocks: u64,
    pub bavail: u64,
    pub bfree: u64,
}

impl Stats {
    pub fn from(mount_point: &Path) -> Result<Option<Self>> {
        let c_mount_point = CString::new(mount_point.as_os_str().as_bytes()).unwrap();
        unsafe {
            let mut statvfs = mem::MaybeUninit::<libc::statvfs>::uninit();
            let code = libc::statvfs(c_mount_point.as_ptr(), statvfs.as_mut_ptr());
            match code {
                0 => {
                    // good
                    let statvfs = statvfs.assume_init();
                    Ok(Some(Stats {
                        bsize: statvfs.f_bsize,
                        blocks: statvfs.f_blocks,
                        bavail: statvfs.f_bavail,
                        bfree: statvfs.f_bfree,
                    }))
                }
                -1 => {
                    // the filesystem wasn't found, it's a strange one, for example a
                    // docker one
                    Ok(None)
                }
                _ => {
                    // unexpected
                    Err(Error::UnexpectedStavfsReturn {
                        code,
                        path: mount_point.to_path_buf(),
                    })
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
