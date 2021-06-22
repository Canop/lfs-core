use super::*;

/// A mount point
#[derive(Debug, Clone)]
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
    let bd_list = BlockDeviceList::read()?;
    read_mountinfo()?
        .drain(..)
        .map(|info| {
            let top_bd = bd_list.find_top(info.dev);
            let disk = top_bd.map(|bd| Disk::new(bd.name.clone()));
            let stats = Stats::from(&info.mount_point)?;
            Ok(Mount { info, disk, stats })
        })
        .collect()
}
