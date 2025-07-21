use {
    super::*,
    crate::*,
    snafu::prelude::*,
    std::{
        fs,
        path::{
            Path,
            PathBuf,
        },
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

    /// a name for a /dev/mapper/ device
    pub dm_name: Option<String>,

    pub id: DeviceId,

    pub parent: Option<DeviceId>,
}

impl BlockDeviceList {
    pub fn read() -> Result<Self, Error> {
        let mut list = Vec::new();
        let root = PathBuf::from("/sys/block");
        append_child_block_devices(None, &root, &mut list, 0)?;
        Ok(Self { list })
    }
    pub fn find_by_id(
        &self,
        id: DeviceId,
    ) -> Option<&BlockDevice> {
        self.list.iter().find(|bd| bd.id == id)
    }
    pub fn find_by_dm_name(
        &self,
        dm_name: &str,
    ) -> Option<&BlockDevice> {
        self.list
            .iter()
            .find(|bd| bd.dm_name.as_ref().is_some_and(|s| s == dm_name))
    }
    pub fn find_by_name(
        &self,
        name: &str,
    ) -> Option<&BlockDevice> {
        self.list.iter().find(|bd| bd.name == name)
    }
    pub fn find_top(
        &self,
        id: DeviceId,
        dm_name: Option<&str>,
        name: Option<&str>,
    ) -> Option<&BlockDevice> {
        self.find_by_id(id)
            .or_else(|| dm_name.and_then(|dm_name| self.find_by_dm_name(dm_name)))
            .or_else(|| name.and_then(|name| self.find_by_name(name)))
            .and_then(|bd| match bd.parent {
                Some(parent_id) => self.find_top(parent_id, None, None),
                None => Some(bd),
            })
    }
}

fn append_child_block_devices(
    parent: Option<DeviceId>,
    parent_path: &Path,
    list: &mut Vec<BlockDevice>,
    depth: usize,
) -> Result<(), Error> {
    let children = fs::read_dir(parent_path).with_context(|_| CantReadDirSnafu {
        path: parent_path.to_path_buf(),
    })?;
    for e in children.flatten() {
        let device_id = fs::read_to_string(e.path().join("dev"))
            .ok()
            .and_then(|s| DeviceId::from_str(s.trim()).ok());
        if let Some(id) = device_id {
            if list.iter().any(|bd| bd.id == id) {
                // already present, probably because of a cycling link
                continue;
            }
            let name = e.file_name().to_string_lossy().to_string();
            let dm_name = sys::read_file(format!("/sys/block/{name}/dm/name"))
                .ok()
                .map(|s| s.trim().to_string());
            list.push(BlockDevice {
                name,
                dm_name,
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
