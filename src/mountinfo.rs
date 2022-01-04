use {
    crate::*,
    lazy_regex::*,
    snafu::prelude::*,
    std::{
        path::PathBuf,
        str::FromStr,
    },
};

/// An id of a mount
pub type MountId = u32;

/// A mount point as described in /proc/self/mountinfo
#[derive(Debug, Clone)]
pub struct MountInfo {
    pub id: MountId,
    pub parent: MountId,
    pub dev: DeviceId,
    pub root: PathBuf,
    pub mount_point: PathBuf,
    pub fs: String,
    pub fs_type: String,
    /// whether it's a bound mount (usually mirroring part of another device)
    pub bound: bool,
}

#[derive(Debug, Snafu)]
#[snafu(display("Could not parse {line} as mount info"))]
pub struct ParseMountInfoError {
    line: String,
}

impl FromStr for MountInfo {
    type Err = ParseMountInfoError;
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        (|| {
            // this parsing is based on `man 5 proc`
            let mut tokens = line.split_whitespace();
            let id = tokens.next()?.parse().ok()?;
            let parent = tokens.next()?.parse().ok()?;
            let dev = tokens.next()?.parse().ok()?;
            let root = str_to_pathbuf(tokens.next()?);
            let mount_point = str_to_pathbuf(tokens.next()?);
            let fs_type = loop {
                let token = tokens.next()?;
                if token != "-" {
                    break token;
                }
            };
            let fs_type = fs_type.to_string();
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
        })().with_context(|| ParseMountInfoSnafu { line })
    }
}

impl MountInfo {
    /// return `<name>` when the path is `/dev/mapper/<name>`
    pub fn dm_name(&self) -> Option<&str> {
        regex_captures!(r#"^/dev/mapper/([^/]+)$"#, &self.fs)
            .map(|(_, dm_name)| dm_name)
    }
    /// return the last token of the fs path
    pub fn fs_name(&self) -> Option<&str> {
        regex_find!(r#"[^\\/]+$"#, &self.fs)
    }
}

/// convert a string to a pathbuf, converting ascii-octal encoded
/// chars.
/// This is necessary because some chars are encoded. For example
/// the `/media/dys/USB DISK` is present as `/media/dys/USB\040DISK`
fn str_to_pathbuf(s: &str) -> PathBuf {
    PathBuf::from(sys::decode_string(s))
}

/// read all the mount points
pub fn read_mountinfo() -> Result<Vec<MountInfo>, Error> {
    let mut mounts: Vec<MountInfo> = Vec::new();
    let path = "/proc/self/mountinfo";
    let file_content = sys::read_file(path)
        .context(CantReadDirSnafu { path })?;
    for line in file_content.trim().split('\n') {
        let mut mount: MountInfo = line.parse()
            .map_err(|source| Error::ParseMountInfo { source })?;
        mount.bound = mounts.iter().any(|m| m.dev == mount.dev);
        mounts.push(mount);
    }
    Ok(mounts)
}

#[test]
fn test_from_str() {
    let mi = MountInfo::from_str(
        "47 21 0:41 / /dev/hugepages rw,relatime shared:27 - hugetlbfs hugetlbfs rw,pagesize=2M"
    ).unwrap();
    assert_eq!(mi.id, 47);
    assert_eq!(mi.dev, DeviceId::new(0, 41));
    assert_eq!(mi.root, PathBuf::from("/"));
    assert_eq!(mi.mount_point, PathBuf::from("/dev/hugepages"));
}
