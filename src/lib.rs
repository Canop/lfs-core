/*!

Use `read_all` to get information on all mounted volumes on a unix system.

```
// get all mount points
let mut mounts = lfs_core::read_all().unwrap();
// only keep the one with a size
mounts.retain(|m| m.size() > 0);
// print them
for mount in mounts {
    dbg!(mount);
}
```
*/

mod device_id;
mod error;
mod mount;
mod sys;

pub use {
    device_id::DeviceId,
    error::{Error, Result},
    mount::{read_all, Mount, MountId, Stats},
};
