/// lfs error type
#[derive(Debug, snafu::Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {

    #[snafu(display("Could not read file {path:?}"))]
    CantReadFile {
        source: std::io::Error,
        path: std::path::PathBuf,
    },

    #[snafu(display("Could not read dir {path:?}"))]
    CantReadDir {
        source: std::io::Error,
        path: std::path::PathBuf,
    },

    #[snafu(display("Could not parse mountinfo"))]
    ParseMountInfo {
        source: crate::ParseMountInfoError,
    },

    #[snafu(display("Unexpected format"))]
    UnexpectedFormat,
}

