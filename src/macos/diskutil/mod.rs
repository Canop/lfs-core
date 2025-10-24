mod diskutil_exec;
mod diskutil_read;

use {
    crate::*,
    snafu::prelude::*,
    std::{
        fs,
        os::unix::fs::MetadataExt,
    },
};

#[derive(Debug)]
struct DuDevice {
    id: String,                    // ex: "disk3s3s1"
    node: String,                  // ex: "/dev/disk3s3s1"
    file_system: Option<String>,   // ex: "APFS"
    mount_point: Option<String>,   // ex: "/"
    part_of_whole: Option<String>, // ex: "disk3"
    protocol: Option<String>,
    removable: Option<bool>,
    read_only: Option<bool>,
    solid_state: Option<bool>,
    encrypted: Option<bool>,
    volume_total_space: Option<u64>,
    volume_used_space: Option<u64>,
    volume_free_space: Option<u64>,
    container_total_space: Option<u64>,
    container_free_space: Option<u64>,
    allocation_block_size: Option<u64>,
    uuid: Option<String>,
    part_uuid: Option<String>,
}

impl DuDevice {
    pub fn stats(&self) -> Option<Stats> {
        let bsize = self.allocation_block_size?;
        let total = self.volume_total_space.or(self.container_total_space)?;
        let blocks = total / bsize;
        let bused = self.volume_used_space? / bsize;
        let free = self.volume_free_space.or(self.container_free_space)?;
        let bfree = free / bsize;
        let bavail = bfree;
        Some(Stats {
            bsize,
            blocks,
            bused,
            bfree,
            bavail,
            inodes: None, // TODO
        })
    }
}

/// Get the device id from the BSD device node
///
/// eg /dev/disk3s4 -> 1:13
fn query_device_id(device_node: &str) -> Result<DeviceId, Error> {
    let stat =
        fs::metadata(device_node).with_context(|_| CantReadFileSnafu { path: device_node })?;
    let rdev = stat.rdev();
    let device_id = DeviceId::from(rdev);
    Ok(device_id)
}

/// Read all the mount points and load basic information on them
pub fn read_mounts(_options: &ReadOptions) -> Result<Vec<Mount>, Error> {
    let devs = diskutil_read::mounted_du_devices()?;
    let mut mounts = Vec::new();
    for dev in devs {
        let stats = dev.stats().ok_or(StatsError::Unreachable);
        let DuDevice {
            id,
            node,
            file_system,
            part_of_whole,
            read_only,
            encrypted,
            protocol,
            mount_point,
            removable,
            solid_state,
            uuid,
            part_uuid,
            ..
        } = dev;
        let Some(mount_point) = mount_point else {
            continue;
        };
        let Some(file_system) = file_system else {
            continue;
        };
        let image = matches!(protocol.as_deref(), Some("Disk Image"));
        let disk = Disk {
            name: part_of_whole.as_ref().unwrap_or(&id).to_string(),
            rotational: solid_state.map(|s| !s),
            removable,
            image,
            read_only,
            ram: false,
            lvm: false,
            crypted: encrypted.unwrap_or(false),
        };
        let dev = query_device_id(&node)?;
        let mut info = MountInfo {
            id: None,
            parent: None,
            dev,
            root: mount_point.clone().into(), // unsure
            mount_point: mount_point.into(),
            options: Default::default(),
            fs: node,
            fs_type: file_system,
            bound: false, // FIXME unsure (as for root)
        };
        if let Some(shortened) = info.fs_type.strip_prefix("MS-DOS ") {
            info.fs_type = shortened.to_string();
        }
        let mount = Mount {
            info,
            fs_label: None, // TODO
            disk: Some(disk),
            stats,
            uuid,
            part_uuid,
        };
        mounts.push(mount);
    }
    Ok(mounts)
}
