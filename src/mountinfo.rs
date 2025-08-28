use {
    crate::*,
    lazy_regex::*,
    snafu::prelude::*,
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
    pub fs: String, // rename into "node" ?
    pub fs_type: String,
    /// whether it's a bound mount (usually mirroring part of another device)
    pub bound: bool,
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
}

#[derive(Debug, Snafu)]
#[snafu(display("Could not parse {line} as mount info"))]
pub struct ParseMountInfoError {
    line: String,
}

#[cfg(target_os = "linux")]
impl std::str::FromStr for MountInfo {
    type Err = ParseMountInfoError;
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        (|| {
            // this parsing is based on `man 5 proc`
            let mut tokens = line.split_whitespace();

            let id = tokens.next()?.parse().ok()?;
            let parent = tokens.next()?.parse().ok()?;

            // while linux mountinfo need an id and a parent id, they're optional in
            // the more global model
            let id = Some(id);
            let parent = Some(parent);

            let dev = tokens.next()?.parse().ok()?;
            let root = str_to_pathbuf(tokens.next()?);
            let mount_point = str_to_pathbuf(tokens.next()?);
            loop {
                let token = tokens.next()?;
                if token == "-" {
                    break;
                }
            }
            let fs_type = tokens.next()?.to_string();
            let fs = tokens.next()?.to_string();
            Some(Self {
                id,
                parent,
                dev,
                root,
                mount_point,
                fs,
                fs_type,
                bound: false, // determined by post-treatment
            })
        })()
        .with_context(|| ParseMountInfoSnafu { line })
    }
}

/// convert a string to a pathbuf, converting ascii-octal encoded
/// chars.
/// This is necessary because some chars are encoded. For example
/// the `/media/dys/USB DISK` is present as `/media/dys/USB\040DISK`
#[cfg(target_os = "linux")]
fn str_to_pathbuf(s: &str) -> PathBuf {
    PathBuf::from(sys::decode_string(s))
}

/// read all the mount points
#[cfg(target_os = "linux")]
pub fn read_mountinfo() -> Result<Vec<MountInfo>, Error> {
    let mut mounts: Vec<MountInfo> = Vec::new();
    let path = "/proc/self/mountinfo";
    let file_content = sys::read_file(path).context(CantReadDirSnafu { path })?;
    for line in file_content.trim().split('\n') {
        let mut mount: MountInfo = line
            .parse()
            .map_err(|source| Error::ParseMountInfo { source })?;
        mount.bound = mounts.iter().any(|m| m.dev == mount.dev);
        mounts.push(mount);
    }
    Ok(mounts)
}

#[cfg(target_os = "linux")]
#[test]
fn test_from_str() {
    use std::str::FromStr;
    let mi = MountInfo::from_str(
        "47 21 0:41 / /dev/hugepages rw,relatime shared:27 - hugetlbfs hugetlbfs rw,pagesize=2M",
    )
    .unwrap();
    assert_eq!(mi.id, Some(47));
    assert_eq!(mi.dev, DeviceId::new(0, 41));
    assert_eq!(mi.root, PathBuf::from("/"));
    assert_eq!(mi.mount_point, PathBuf::from("/dev/hugepages"));

    let mi = MountInfo::from_str(
        "106 26 8:17 / /home/dys/dev rw,relatime shared:57 - xfs /dev/sdb1 rw,attr2,inode64,noquota"
    ).unwrap();
    assert_eq!(mi.id, Some(106));
    assert_eq!(mi.dev, DeviceId::new(8, 17));
    assert_eq!(&mi.fs, "/dev/sdb1");
    assert_eq!(&mi.fs_type, "xfs");
}
