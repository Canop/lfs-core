use crate::*;

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
    /// tell whether the mount looks remote
    ///
    /// Heuristics copied from https://github.com/coreutils/gnulib/blob/master/lib/mountlist.c
    #[cfg(unix)]
    pub fn is_remote(&self) -> bool {
        self.info.is_remote()
    }

    #[cfg(windows)]
    pub fn is_remote(&self) -> bool {
        self.disk.as_ref().is_some_and(|disk| disk.remote)
    }
}
