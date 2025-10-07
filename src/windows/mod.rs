mod volume;

use crate::{
    Error,
    Mount,
    ReadOptions,
    windows::volume::get_volumes,
};

/// Read all the mount points and load basic information on them
pub fn read_mounts(_options: &ReadOptions) -> Result<Vec<Mount>, Error> {
    Ok(get_volumes()?
        .into_iter()
        .flat_map(|volume| volume.to_dysk_mounts().ok())
        .flatten()
        .collect())
}
