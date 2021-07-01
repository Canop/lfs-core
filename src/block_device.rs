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
        append_child_block_devices(None, &root, &mut list, 0)?;
        Ok(Self { list })
    }
    pub fn find_by_id(&self, id: DeviceId) -> Option<&BlockDevice> {
        self.list
            .iter()
            .find(|bd| bd.id == id)
    }
    pub fn find_by_name(&self, name: &str) -> Option<&BlockDevice> {
        self.list
            .iter()
            .find(|bd| bd.name == name)
    }
    pub fn find_top(
        &self,
        id: DeviceId,
        name: Option<&str>,
    ) -> Option<&BlockDevice> {
        self.find_by_id(id)
            .or_else(|| name.and_then(|name| self.find_by_name(name)))
            .and_then(|bd| {
                match bd.parent {
                    Some(parent_id) => self.find_top(parent_id, None),
                    None => Some(bd),
                }
            })
    }
}


fn append_child_block_devices(
    parent: Option<DeviceId>,
    parent_path: &Path,
    list: &mut Vec<BlockDevice>,
    depth: usize,
) -> Result<()> {
    for e in fs::read_dir(parent_path)?.flatten() {
        let device_id = fs::read_to_string(e.path().join("dev")).ok()
            .and_then(|s| DeviceId::from_str(s.trim()).ok());
        if let Some(id) = device_id {
            if list.iter().find(|bd| bd.id == id).is_some() {
                // already present, probably because of a cycling link
                continue;
            }
            list.push(BlockDevice {
                name: e.file_name().to_string_lossy().to_string(),
                id,
                parent,
            });
            if depth > 15 {
                // there's probably a link cycle
                continue;
            }
            append_child_block_devices(Some(id), &e.path(), list, depth + 1)?;
        }
    }
    Ok(())
}


