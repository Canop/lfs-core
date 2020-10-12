use {
    std::{
        fs::File,
        io::{self, Read},
        path::Path,
    },
};

/// read a system file into a string
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path.as_ref())?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

