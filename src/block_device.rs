use {
    super::*,
    std::{
        fs,
        path::{Path, PathBuf},
        str::FromStr,
    },
};

/// the list of all found block devices
#[derive(Debug, Clone)]
pub struct BlockDeviceList {
    list: Vec<BlockDevice>,
}

/// a "block device", that is a device listed in
///  the /sys/block tree with a device id
#[derive(Debug, Clone)]
pub struct BlockDevice {
    pub name: String,
    pub id: DeviceId,
    pub parent: Option<DeviceId>,
}

impl BlockDeviceList {
    pub fn read() -> Result<Self> {
        let mut list = Vec::new();
        let root = PathBuf::from("/sys/block");
        append_child_block_devices(None, &root, &mut list)?;
        Ok(Self { list })
    }
    pub fn find(&self, id: DeviceId) -> Option<&BlockDevice> {
        self.list
            .iter()
            .find(|bd| bd.id == id)
    }
    pub fn find_top(&self, id: DeviceId) -> Option<&BlockDevice> {
        self.find(id)
            .and_then(|bd| {
                match bd.parent {
                    Some(parent_id) => self.find_top(parent_id),
                    None => Some(bd),
                }
            })
    }
}


fn append_child_block_devices(
    parent: Option<DeviceId>,
    parent_path: &Path,
    list: &mut Vec<BlockDevice>,
) -> Result<()> {
    for e in fs::read_dir(parent_path)?.flatten() {
        let device_id = fs::read_to_string(e.path().join("dev")).ok()
            .and_then(|s| DeviceId::from_str(s.trim()).ok());
        if let Some(id) = device_id {
            list.push(BlockDevice {
                name: e.file_name().to_string_lossy().to_string(),
                id,
                parent,
            });
            append_child_block_devices(Some(id), &e.path(), list)?;
        }
    }
    Ok(())
}


