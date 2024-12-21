use super::*;

/// A mount point
#[derive(Debug, Clone)]
pub struct Mount {
    pub info: MountInfo,
    pub fs_label: Option<String>,
    pub disk: Option<Disk>,
    pub stats: Result<Stats, StatsError>,
    pub uuid: Option<String>,
    pub part_uuid: Option<String>,
}

impl Mount {
    /// Return inodes information, when available and consistent
    pub fn inodes(&self) -> Option<&Inodes> {
        self.stats
            .as_ref()
            .ok()
            .and_then(|stats| stats.inodes.as_ref())
    }
    /// Return the stats, if they could be fetched and
    /// make sense.
    ///
    /// Most often, you don't care *why* there are no stats,
    /// because the error cases are mostly non storage volumes,
    /// so it's a best practice to no try to analyze the error
    /// but just use this option returning method.
    ///
    /// The most interesting case is when a network volume is
    /// unreachable, which you can test with is_unreachable().
    pub fn stats(&self) -> Option<&Stats> {
        self.stats.as_ref().ok()
    }
    /// Tell whether the reason we have no stats is because the
    /// filesystem is unreachable
    pub fn is_unreachable(&self) -> bool {
        matches!(self.stats, Err(StatsError::Unreachable))
    }
}

#[derive(Debug, Clone)]
pub struct ReadOptions {
    remote_stats: bool,
}
impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            remote_stats: true,
        }
    }
}
impl ReadOptions {
    pub fn remote_stats(&mut self, v: bool) {
        self.remote_stats = v;
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
            let top_bd = bd_list.find_top(
                info.dev,
                info.dm_name(),
                info.fs_name(),
            );
            let fs_label = get_label(&info.fs, by_label.as_deref());
            let uuid = get_label(&info.fs, by_uuid.as_deref());
            let part_uuid = get_label(&info.fs, by_partuuid.as_deref());
            let disk = top_bd.map(|bd| Disk::new(bd.name.clone()));
            let stats = if !options.remote_stats && info.is_remote() {
                Err(StatsError::Excluded)
            } else {
                Stats::from(&info.mount_point)
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
