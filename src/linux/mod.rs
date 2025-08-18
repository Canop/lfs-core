mod block_device;

use {
    crate::*,
    block_device::*,
    lazy_regex::*,
    std::{
        ffi::CString,
        mem,
        os::unix::ffi::OsStrExt,
        path::Path,
    },
};

pub fn new_disk(name: String) -> Disk {
    let rotational = sys::read_file_as_bool(format!("/sys/block/{name}/queue/rotational"));
    let removable = sys::read_file_as_bool(format!("/sys/block/{name}/removable"));
    let ram = regex_is_match!(r#"^zram\d*$"#, &name);
    let dm_uuid = sys::read_file(format!("/sys/block/{name}/dm/uuid")).ok();
    let crypted = dm_uuid
        .as_ref()
        .is_some_and(|uuid| uuid.starts_with("CRYPT-"));
    let lvm = dm_uuid.is_some_and(|uuid| uuid.starts_with("LVM-"));
    Disk {
        name,
        rotational,
        removable,
        image: false,
        read_only: None,
        ram,
        lvm,
        crypted,
    }
}

/// Read all the mount points and load basic information on them
pub fn read_mounts(options: &ReadOptions) -> Result<Vec<Mount>, Error> {
    let by_label = read_by("label").ok();
    let by_uuid = read_by("uuid").ok();
    let by_partuuid = read_by("partuuid").ok();

    // we'll find the disk for a filesystem by taking the longest
    // disk whose name starts the one of our partition
    // hence the sorting.
    let bd_list = BlockDeviceList::read()?;
    read_mountinfo()?
        .drain(..)
        .map(|info| {
            let top_bd = bd_list.find_top(info.dev, info.dm_name(), info.fs_name());
            let fs_label = get_label(&info.fs, by_label.as_deref());
            let uuid = get_label(&info.fs, by_uuid.as_deref());
            let part_uuid = get_label(&info.fs, by_partuuid.as_deref());
            let disk = top_bd.map(|bd| new_disk(bd.name.clone()));
            let stats = if !options.remote_stats && info.is_remote() {
                Err(StatsError::Excluded)
            } else {
                read_stats(&info.mount_point)
            };
            Ok(Mount {
                info,
                fs_label,
                disk,
                stats,
                uuid,
                part_uuid,
            })
        })
        .collect()
}

pub fn read_stats(mount_point: &Path) -> Result<Stats, StatsError> {
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
                #[allow(clippy::useless_conversion)]
                let inodes = Inodes::new(files.into(), ffree.into(), favail.into());

                #[allow(clippy::useless_conversion)]
                Ok(Stats {
                    bsize: bsize.into(),
                    blocks: blocks.into(),
                    bused: bused.into(),
                    bfree: bfree.into(),
                    bavail: bavail.into(),
                    inodes: inodes.into(),
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
