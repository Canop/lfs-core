/*!

Use `lfs_core::read_mounts` to get information on all mounted volumes on a unix system.

```
// get all mount points
let options = lfs_core::ReadOptions::default();
let mut mounts = lfs_core::read_mounts(&options).unwrap();
// only keep the one with size stats
mounts.retain(|m| m.stats.is_ok());
// print them
for mount in mounts {
    dbg!(mount);
}
```

The [dysk](https://github.com/Canop/dysk) application is a viewer for lfs-core and shows you the information you're expected to find in mounts.

*/

mod device_id;
mod disk;
mod error;
mod inodes;
mod label;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
mod mount;
mod mountinfo;
mod read_options;
mod stats;
mod sys;
#[cfg(target_os = "windows")]
mod windows;

pub use {
    device_id::*,
    disk::*,
    error::*,
    inodes::*,
    label::*,
    mount::*,
    mountinfo::*,
    read_options::*,
    stats::*,
};

#[cfg(target_os = "linux")]
pub use linux::read_mounts;
#[cfg(target_os = "macos")]
pub use macos::read_mounts;
#[cfg(target_os = "windows")]
pub use windows::read_mounts;
