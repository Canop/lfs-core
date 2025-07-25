use {
    crate::error::*,
    lazy_regex::*,
    snafu::prelude::*,
    std::process,
};

pub fn du_lines(args: &[&str]) -> Result<Vec<String>, Error> {
    let exe = "diskutil";
    let output = process::Command::new(exe)
        .args(args)
        .output()
        .with_context(|_| CantExecuteSnafu { exe })?;
    let output = str::from_utf8(&output.stdout).map_err(|_| Error::UnexpectedFormat)?;
    let lines = output.lines().map(|s| s.to_string()).collect();
    Ok(lines)
}

/// return all container/volume identifiers, which are strings like "disk6" or "disk3s3s3"
#[allow(dead_code)]
pub fn du_identifiers() -> Result<Vec<String>, Error> {
    let mut ids = Vec::new();
    for line in du_lines(&["list"])? {
        let Some((_, id)) = regex_captures!(r"^\s+\d+:.+\s\wB\s+(disk\w+)$", &line) else {
            continue;
        };
        ids.push(id.to_string());
    }
    Ok(ids)
}
