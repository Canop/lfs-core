use {
    crate::*,
    lazy_regex::*,
    std::path::PathBuf,
};

static REMOTE_ONLY_FS_TYPES: &[&str] = &[
    "afs",
    "coda",
    "auristorfs",
    "fhgfs",
    "gpfs",
    "ibrix",
    "ocfs2",
    "vxfs",
];

/// An id of a mount
pub type MountId = u32;

/// A mount point as described in /proc/self/mountinfo
#[derive(Debug, Clone)]
pub struct MountInfo {
    pub id: Option<MountId>,
    pub parent: Option<MountId>,
    pub dev: DeviceId,
    pub root: PathBuf,
    pub mount_point: PathBuf,
    pub options: Vec<MountOption>,
    pub fs: String, // rename into "node" ?
    pub fs_type: String,
    /// whether it's a bound mount (usually mirroring part of another device)
    pub bound: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MountOption {
    pub name: String,
    pub value: Option<String>,
}

impl MountOption {
    pub fn new<S: Into<String>>(
        name: S,
        value: Option<S>,
    ) -> Self {
        MountOption {
            name: name.into(),
            value: value.map(|s| s.into()),
        }
    }
}

impl MountInfo {
    /// return `<name>` when the path is `/dev/mapper/<name>`
    pub fn dm_name(&self) -> Option<&str> {
        regex_captures!(r#"^/dev/mapper/([^/]+)$"#, &self.fs).map(|(_, dm_name)| dm_name)
    }
    /// return the last token of the fs path
    pub fn fs_name(&self) -> Option<&str> {
        regex_find!(r#"[^\\/]+$"#, &self.fs)
    }
    /// tell whether the mount looks remote
    ///
    /// Heuristics copied from https://github.com/coreutils/gnulib/blob/master/lib/mountlist.c
    pub fn is_remote(&self) -> bool {
        self.fs.contains(':')
            || (self.fs.starts_with("//")
                && ["cifs", "smb3", "smbfs"].contains(&self.fs_type.as_ref()))
            || REMOTE_ONLY_FS_TYPES.contains(&self.fs_type.as_ref())
            || self.fs == "-hosts"
    }
    /// return a string like "rw,noatime,compress=zstd:3,space_cache=v2,subvolid=256"
    /// (as in /proc/mountinfo)
    pub fn options_string(&self) -> String {
        let mut s = String::new();
        let mut first = true;
        for option in &self.options {
            if !first {
                s.push(',');
            }
            s.push_str(&option.name);
            if let Some(value) = &option.value {
                s.push('=');
                s.push_str(value);
            }
            first = false;
        }
        s
    }
    /// tell whether the option (eg "compress", "rw", "noatime") is present
    /// among options
    pub fn has_option(
        &self,
        name: &str,
    ) -> bool {
        for option in &self.options {
            if option.name == name {
                return true;
            }
        }
        false
    }
    /// return the value of the mountoption, or None
    pub fn option_value(
        &self,
        name: &str,
    ) -> Option<&str> {
        for option in &self.options {
            if option.name == name {
                return option.value.as_deref();
            }
        }
        None
    }
}
