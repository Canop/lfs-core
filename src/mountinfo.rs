use {
    crate::*,
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
        let root = next(tokens)?.into();
        let mount_point = PathBuf::from(next(tokens)?);
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
        })
    }
}

fn next<'a, 'b>(split: &'b mut SplitWhitespace<'a>) -> Result<&'a str> {
    split.next().ok_or(Error::UnexpectedFormat)
}
fn skip_until<'a, 'b>(split: &'b mut SplitWhitespace<'a>, sep: &'static str) -> Result<()> {
    Ok(loop {
        if next(split)? == sep {
            break;
        }
    })
}

/// read all the mount points and load basic information on them
pub fn read_mountinfo() -> Result<Vec<MountInfo>> {
    sys::read_file("/proc/self/mountinfo")?
        .trim()
        .split('\n')
        .map(str::parse)
        .inspect(|r| {
            if let Err(e) = r {
                eprintln!("Error while parsing a mount line: {}", e);
            }
        })
        //.filter(Result::is_ok)
        .collect()
}
