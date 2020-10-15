use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

/// read a system file into a string
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path.as_ref())?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

/// read a system file into a boolean (assuming "0" or "1")
pub fn read_file_as_bool<P: AsRef<Path>>(path: P) -> Option<bool> {
    read_file(path)
        .ok()
        .and_then(|c| {
            match c.trim().as_ref() {
                "0" => Some(false),
                "1" => Some(true),
                _ => None,
            }
        })
}
