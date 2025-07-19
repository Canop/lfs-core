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

The [lfs](https://github.com/Canop/lfs) application is a viewer for lfs-core and shows you the information you're expected to find in mounts.

*/

mod disk;
mod error;
mod inodes;
mod label;
#[cfg(target_os="linux")] mod linux;
#[cfg(target_os="macos")] mod macos;
mod mount;
mod mountinfo;
mod stats;
mod sys;

pub use {
    disk::*,
    error::*,
    inodes::*,
    label::*,
    mount::*,
    mountinfo::*,
    stats::*,
};

#[cfg(target_os="linux")]
pub use linux::{
    read_mounts,
    DeviceId,
};
#[cfg(target_os="macos")]
pub use macos::{
    read_mounts,
    DeviceId,
};

