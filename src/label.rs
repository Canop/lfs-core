use {
    super::*,
    snafu::prelude::*,
    std::fs,
};

/// the labelling of a file-system, that
/// is the pair (label, fs)
#[derive(Debug, Clone)]
pub struct Labelling {
    pub label: String,
    pub fs_name: String,
}

pub fn get_label(
    fs_name: &str,
    labellings: Option<&[Labelling]>,
) -> Option<String> {
    labellings.as_ref().and_then(|labels| {
        labels
            .iter()
            .find(|label| label.fs_name == fs_name)
            .map(|label| label.label.clone())
    })
}

/// try to read all mappings defined in /dev/disk/by-<by_kind>,
/// where by_kind is one of "label", "uuid", "partuuid", "diskseq", etc.
///
/// An error can't be excluded as not all systems expose
/// this information the way lfs-core reads it.
pub fn read_by(by_kind: &str) -> Result<Vec<Labelling>, Error> {
    let path = format!("/dev/disk/by-{by_kind}");
    let entries = fs::read_dir(&path).context(CantReadDirSnafu { path })?;
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
            let fs_name = format!("/dev/{}", linked_path.strip_prefix("../../")?,);
            Some(Labelling { label, fs_name })
        })
        .collect();
    Ok(labels)
}
