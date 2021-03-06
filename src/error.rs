/// lfs error type
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Unexpected format")]
    UnexpectedFormat,
    #[error("Parse int error: {0}")]
    NotAnInt(#[from] std::num::ParseIntError),
    #[error("libc.statvfs({path:?}) returned {code}")]
    UnexpectedStavfsReturn { code: i32, path: std::path::PathBuf },
}

pub type Result<T> = std::result::Result<T, Error>;
