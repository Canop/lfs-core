use {
    crate::*,
    libc::{
        MNT_NOWAIT,
        getfsstat,
        statfs,
    },
};

/// Basically data coming from getfsstat (BSD style)
#[derive(Debug)]
pub struct DevMountInfo {
    pub device: String,
    pub dev: DeviceId,
    pub mount_point: String,
    pub fs_type: String,
    pub stats: Stats,
    pub options: Vec<MountOption>,
}

impl DevMountInfo {
    pub fn to_mount_info(&self) -> MountInfo {
        MountInfo {
            id: None,
            parent: None,
            dev: self.dev,
            root: self.mount_point.clone().into(),
            mount_point: self.mount_point.clone().into(),
            options: self.options.clone(),
            fs: self.device.clone(),
            fs_type: self.fs_type.clone(),
            bound: false,
        }
    }
    pub fn get_all() -> Vec<Self> {
        unsafe {
            // First call to get the number of filesystems
            let count = getfsstat(std::ptr::null_mut(), 0, MNT_NOWAIT);
            if count <= 0 {
                return Vec::new();
            }

            // Allocate buffer
            let mut buf: Vec<statfs> = Vec::with_capacity(count as usize);
            let buf_size = (count as usize) * std::mem::size_of::<statfs>();

            // Second call to get the data
            let actual_count = getfsstat(buf.as_mut_ptr(), buf_size as i32, MNT_NOWAIT);
            if actual_count <= 0 {
                return Vec::new();
            }
            buf.set_len(actual_count as usize);

            buf.into_iter()
                .filter_map(|stat| {
                    let device = std::ffi::CStr::from_ptr(stat.f_mntfromname.as_ptr())
                        .to_str()
                        .ok()?;
                    let fsid: u64 = std::mem::transmute_copy(&stat.f_fsid);
                    let dev = fsid.into();
                    let mount_point = std::ffi::CStr::from_ptr(stat.f_mntonname.as_ptr())
                        .to_str()
                        .ok()?;
                    let fs_type = std::ffi::CStr::from_ptr(stat.f_fstypename.as_ptr())
                        .to_str()
                        .ok()?;
                    let stats = Stats {
                        bsize: stat.f_bsize as u64,
                        blocks: stat.f_blocks,
                        bfree: stat.f_bfree,
                        bavail: stat.f_bavail,
                        bused: stat.f_blocks - stat.f_bavail,
                        inodes: None,
                    };
                    let fs_type = match fs_type {
                        "apfs" => "APFS",
                        "exfat" => "ExFAT",
                        "ftp" => "FTP",
                        "hfs" => "HFS+",
                        "msdos" if stats.bsize * stats.blocks > 2_147_484_648 => "FAT32",
                        "msdos" => "FAT", // will be detemined using device.content
                        "nfs" => "NFS",
                        "ntfs" => "NTFS",
                        "udf" => "UDF",
                        "ufs" => "UFS",
                        "xfs" => "XHS",
                        "zfs" => "ZFS",
                        v => v, // other ones unchanged
                    };
                    // we'll try to build a "mount options" array consistent with the semantics of linux
                    // Constants are defined in https://github.com/apple/darwin-xnu/blob/main/bsd/sys/mount.h
                    // I'm not sure how stable those flag values are
                    let flags: u32 = stat.f_flags;
                    let mut options = Vec::new();
                    if flags & 1 == 0 {
                        // MNT_READ_ONLY = 1
                        options.push(MountOption::new("rw", None));
                    }
                    if flags & 2 != 0 {
                        // MNT_SYNCHRONOUS = 2
                        options.push(MountOption::new("synchronous", None));
                    }
                    if flags & 4 != 0 {
                        // MNT_NOEXEC = 4
                        options.push(MountOption::new("noexec", None));
                    }
                    if flags & 8 != 0 {
                        // MNT_NOSUID = 8
                        options.push(MountOption::new("nosuid", None));
                    }
                    if flags & 16 != 0 {
                        // MNT_NODEV = 16
                        options.push(MountOption::new("nodev", None));
                    }
                    if flags & 32 != 0 {
                        // MNT_UNION = 32
                        options.push(MountOption::new("union", None));
                    }
                    if flags & 64 != 0 {
                        // MNT_ASYNC = 64
                        options.push(MountOption::new("async", None));
                    }
                    if flags & 128 != 0 {
                        // MNT_CPROTECT = 128
                        options.push(MountOption::new("cprotect", None));
                    }
                    if flags & 512 != 0 {
                        // MNT_REMOVABLE = 512
                        options.push(MountOption::new("removable", None));
                    }

                    // Following ones don't seem correct
                    // if flags & 0x00100000 != 0 { // MNT_DONTBROWSE  = 0x00100000
                    //     options.push(MountOption::new("dontbrowse", None));
                    // }
                    // if flags & 0x10000000 != 0 { // MNT_NOATIME = 0x10000000
                    //     options.push(MountOption::new("noatime", None));
                    // }

                    Some(DevMountInfo {
                        device: device.to_string(),
                        dev,
                        mount_point: mount_point.to_string(),
                        fs_type: fs_type.to_string(),
                        stats,
                        options,
                    })
                })
                .collect()
        }
    }
}
