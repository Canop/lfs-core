use {
    crate::*,
    lazy_regex::*,
    snafu::prelude::*,
    std::path::PathBuf,
};

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
            // Structure is also visible at
            //  https://man7.org/linux/man-pages/man5/proc_pid_mountinfo.5.html
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

            let direct_options =
                regex_captures_iter!("(?:^|,)([^=,]+)(?:=([^=,]*))?", tokens.next()?,);

            let mut options: Vec<MountOption> = direct_options
                .map(|c| {
                    let name = c.get(1).unwrap().as_str().to_string();
                    let value = c.get(2).map(|v| v.as_str().to_string());
                    MountOption { name, value }
                })
                .collect();

            // skip optional fields in the form name:value where
            // name can be "shared", "master", "propagate_for", or "unbindable"
            loop {
                let token = tokens.next()?;
                if token == "-" {
                    break;
                }
            }

            let fs_type = tokens.next()?.to_string();
            let fs = tokens.next()?.to_string();

            if let Some(super_options) = tokens.next() {
                for c in regex_captures_iter!("(?:^|,)([^=,]+)(?:=([^=,]*))?", super_options) {
                    let name = c.get(1).unwrap().as_str().to_string();
                    if name == "rw" {
                        continue; // rw at super level is not relevant
                    }
                    if options.iter().any(|o| o.name == name) {
                        continue;
                    }
                    let value = c.get(2).map(|v| v.as_str().to_string());
                    options.push(MountOption { name, value });
                }
            }

            Some(Self {
                id,
                parent,
                dev,
                root,
                mount_point,
                options,
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
pub fn read_all_mountinfos() -> Result<Vec<MountInfo>, Error> {
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
#[allow(clippy::bool_assert_comparison)]
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
    assert_eq!(mi.options_string(), "rw,relatime,pagesize=2M".to_string());

    let mi = MountInfo::from_str(
        "106 26 8:17 / /home/dys/dev rw,noatime,compress=zstd:3 shared:57 - btrfs /dev/sdb1 rw,attr2,inode64,noquota"
    ).unwrap();
    assert_eq!(mi.id, Some(106));
    assert_eq!(mi.dev, DeviceId::new(8, 17));
    assert_eq!(&mi.fs, "/dev/sdb1");
    assert_eq!(&mi.fs_type, "btrfs");
    let mut options = mi.options.clone().into_iter();
    assert_eq!(options.next(), Some(MountOption::new("rw", None)),);
    assert_eq!(options.next(), Some(MountOption::new("noatime", None)));
    assert_eq!(
        options.next(),
        Some(MountOption::new("compress", Some("zstd:3")))
    );
    assert_eq!(mi.has_option("noatime"), true);
    assert_eq!(mi.has_option("relatime"), false);
    assert_eq!(mi.option_value("thing"), None);
    assert_eq!(mi.option_value("compress"), Some("zstd:3"));
    assert_eq!(
        mi.options_string(),
        "rw,noatime,compress=zstd:3,attr2,inode64,noquota".to_string()
    );

    let mi = MountInfo::from_str(
        "73 2 0:33 /root / rw,relatime shared:1 - btrfs /dev/vda3 rw,seclabel,compress=zstd:1,ssd,space_cache=v2,subvolid=256,subvol=/root"
    ).unwrap();
    assert_eq!(mi.option_value("compress"), Some("zstd:1"));
    assert_eq!(
        mi.options_string(),
        "rw,relatime,seclabel,compress=zstd:1,ssd,space_cache=v2,subvolid=256,subvol=/root"
            .to_string()
    );
}
