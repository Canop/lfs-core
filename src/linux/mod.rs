mod block_device;
mod device_id;

use {
    crate::*,
    block_device::*,
    device_id::*,
};

pub fn new_disk(name: String) -> Disk {
    let rotational = sys::read_file_as_bool(&format!("/sys/block/{}/queue/rotational", name));
    let removable = sys::read_file_as_bool(&format!("/sys/block/{}/removable", name));
    let ram = regex_is_match!(r#"^zram\d*$"#, &name);
    let dm_uuid = sys::read_file(&format!("/sys/block/{}/dm/uuid", name)).ok();
    let crypted = dm_uuid
        .as_ref()
        .map_or(false, |uuid| uuid.starts_with("CRYPT-"));
    let lvm = dm_uuid.map_or(false, |uuid| uuid.starts_with("LVM-"));
    Disk {
        name,
        rotational,
        removable,
        image: false,
        read_only: None,
        ram,
        lvm,
        crypted,
    }
}

/// Read all the mount points and load basic information on them
pub fn read_mounts(options: &ReadOptions) -> Result<Vec<Mount>, Error> {
    let by_label = read_by("label").ok();
    let by_uuid = read_by("uuid").ok();
    let by_partuuid = read_by("partuuid").ok();

    // we'll find the disk for a filesystem by taking the longest
    // disk whose name starts the one of our partition
    // hence the sorting.
    let bd_list = BlockDeviceList::read()?;
    read_mountinfo()?
        .drain(..)
        .map(|info| {
            let top_bd = bd_list.find_top(info.dev, info.dm_name(), info.fs_name());
            let fs_label = get_label(&info.fs, by_label.as_deref());
            let uuid = get_label(&info.fs, by_uuid.as_deref());
            let part_uuid = get_label(&info.fs, by_partuuid.as_deref());
            let disk = top_bd.map(|bd| Disk::new(bd.name.clone()));
            let stats = if !options.remote_stats && info.is_remote() {
                Err(StatsError::Excluded)
            } else {
                Stats::from(&info.mount_point)
            };
            Ok(Mount {
                info,
                fs_label,
                disk,
                stats,
                uuid,
                part_uuid,
            })
        })
        .collect()
}
