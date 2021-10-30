use {
    super::*,
    std::{
        fs,
    },
};

/// the labelling of a file-system, that
/// is the pair (label, fs)
#[derive(Debug, Clone)]
pub struct Labelling {
    pub label: String,
    pub fs_name: String,
}

/// try to find all file-system labels
///
/// An error can't be excluded as not all systems expose
/// this information the way lfs-core reads it.
pub fn read_labels() -> Result<Vec<Labelling>> {
    let entries = fs::read_dir("/dev/disk/by-label")?;
    let labels = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let md = entry.metadata().ok()?;
            let file_type = md.file_type();
            if !file_type.is_symlink() {
                return None;
            }
            let label = sys::decode_string(entry.file_name().to_string_lossy());
            let linked_path = fs::read_link(entry.path())
                .map(|path| path.to_string_lossy().to_string())
                .ok()?;
            let fs_name = format!(
                "/dev/{}",
                linked_path.strip_prefix("../../")?,
            );
            Some(Labelling { label, fs_name })
        })
        .collect();
    Ok(labels)
}
