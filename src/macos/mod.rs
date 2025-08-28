mod diskutil;
mod iokit;

use crate::*;

/// Read all the mount points and load information on them
pub fn read_mounts(options: &ReadOptions) -> Result<Vec<Mount>, Error> {
    match options.strategy {
        Some(Strategy::Iokit) => {
            eprintln!("Calling IOKit to read device information");
            iokit::read_mounts(options)
        }
        Some(Strategy::Diskutil) => {
            eprintln!("Calling diskutil to read device information");
            diskutil::read_mounts(options)
        }
        _ => iokit::read_mounts(options),
    }
}
