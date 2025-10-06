use crate::{
    Error,
    Mount,
    ReadOptions,
};

/// Read all the mount points and load basic information on them
pub fn read_mounts(_options: &ReadOptions) -> Result<Vec<Mount>, Error> {
    Ok(Vec::new())
}
