use {
    crate::*,
    lazy_regex::*,
    std::{
        path::PathBuf,
        str::{FromStr, SplitWhitespace},
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

impl FromStr for MountInfo {
    type Err = Error;
    fn from_str(line: &str) -> Result<Self> {
        // this parsing is based on `man 5 proc`
        let mut tokens = line.split_whitespace();
        let tokens = &mut tokens;
        let id = next(tokens)?.parse()?;
        let parent = next(tokens)?.parse()?;
        let dev = next(tokens)?.parse()?;
        let root = str_to_pathbuf(next(tokens)?);
        let mount_point = str_to_pathbuf(next(tokens)?);
        skip_until(tokens, "-")?;
        let fs_type = next(tokens)?.to_string();
        let fs = next(tokens)?.to_string();
        Ok(Self {
            id,
            parent,
            dev,
            root,
            mount_point,
            fs,
            fs_type,
            bound: false, // determined by post-treatment
        })
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

fn next<'a, 'b>(split: &'b mut SplitWhitespace<'a>) -> Result<&'a str> {
    split.next().ok_or(Error::UnexpectedFormat)
}
fn skip_until(split: &mut SplitWhitespace, sep: &'static str) -> Result<()> {
    loop {
        if next(split)? == sep {
            break;
        }
    }
    Ok(())
}

/// read all the mount points
pub fn read_mountinfo() -> Result<Vec<MountInfo>> {
    let mut mounts: Vec<MountInfo> = Vec::new();
    for line in sys::read_file("/proc/self/mountinfo")?.trim().split('\n') {
        let mut mount: MountInfo = line.parse()?;
        mount.bound = mounts.iter().any(|m| m.dev == mount.dev);
        mounts.push(mount);
    }
    Ok(mounts)
}
