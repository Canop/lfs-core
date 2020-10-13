use super::*;

/// A mount point
#[derive(Debug)]
pub struct Mount {
    pub info: MountInfo,
    pub disk: Option<Disk>,
    pub stats: Option<Stats>,
}

impl Mount {
    pub fn size(&self) -> u64 {
        self.stats.as_ref().map_or(0, |s| s.size())
    }
}

/// read all the mount points and load basic information on them
pub fn read_mounts() -> Result<Vec<Mount>> {
    // we'll find the disk for a filesystem by taking the longest
    // disk whose name starts the one of our partition
    // hence the sorting.
    let mut disks = read_disks()?;
    disks.sort_by_key(|disk| std::cmp::Reverse(disk.name.len()));
    read_mountinfo()?
        .drain(..)
        .map(|info| {
            let disk = info.fs.strip_prefix("/dev/").and_then(|partition_name| {
                disks
                    .iter()
                    .find(|d| partition_name.starts_with(&d.name))
                    .map(|d| d.clone())
            });
            let stats = Stats::from(&info.mount_point)?;
            Ok(Mount { info, disk, stats })
        })
        .collect()
}
