/*!

Use `lfs_core::read_mounts` to get information on all mounted volumes on a unix system.

```
// get all mount points
let mut mounts = lfs_core::read_mounts().unwrap();
// only keep the one with size stats
mounts.retain(|m| m.stats.is_some());
// print them
for mount in mounts {
    dbg!(mount);
}
```

The [lfs](https://github.com/Canop/lfs) application is a viewer for lfs-core and shows you the information you're expected to find in mounts.

*/

mod block_device;
mod device_id;
mod disk;
mod error;
mod inodes;
mod label;
mod mount;
mod mountinfo;
mod stats;
mod sys;

pub use {
    block_device::*,
    device_id::*,
    disk::*,
    error::*,
    inodes::*,
    label::*,
    mount::*,
    mountinfo::*,
    stats::*,
};
