use {
    lazy_regex::*,
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

/// read a system file into a boolean (assuming "0" or "1")
pub fn read_file_as_bool<P: AsRef<Path>>(path: P) -> Option<bool> {
    read_file(path).ok().and_then(|c| match c.trim() {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    })
}

/// decode ascii-octal or ascii-hexa encoded strings
pub fn decode_string<S: AsRef<str>>(s: S) -> String {
    // replacing octal escape sequences
    let s = regex_replace_all!(r#"\\0(\d\d)"#, s.as_ref(), |_, n: &str| {
        let c = u8::from_str_radix(n, 8).unwrap() as char;
        c.to_string()
    });
    // replacing hexa escape sequences
    let s = regex_replace_all!(r#"\\x([0-9a-fA-F]{2})"#, &s, |_, n: &str| {
        let c = u8::from_str_radix(n, 16).unwrap() as char;
        c.to_string()
    });
    s.to_string()
}
