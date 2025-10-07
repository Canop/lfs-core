#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::*;
#[cfg(windows)]
pub use windows::*;

#[derive(Debug, snafu::Snafu, Clone, Copy, PartialEq, Eq)]
#[snafu(visibility(pub(crate)))]
pub enum StatsError {
    #[snafu(display("Could not stat mount point"))]
    Unreachable,

    #[snafu(display("Unconsistent stats"))]
    Unconsistent,

    /// Options made us not even try
    #[snafu(display("Excluded"))]
    Excluded,
}
