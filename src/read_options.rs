use std::{
    str::FromStr,
    time::Duration,
};

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Strategy {
    /// On mac, with this strategy, IOKit is called
    Iokit,
    /// On mac, with this strategy, the output of the diskutil
    /// command is parsed
    Diskutil,
}

#[derive(Debug, Clone, Copy)]
pub struct ReadOptions {
    pub(crate) remote_stats: bool,
    pub(crate) strategy: Option<Strategy>,
    pub(crate) stats_timeout: Option<Duration>,
}
impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            remote_stats: true,
            strategy: None,
            stats_timeout: Some(Duration::from_millis(100)),
        }
    }
}
impl ReadOptions {
    pub fn remote_stats(
        mut self,
        v: bool,
    ) -> Self {
        self.remote_stats = v;
        self
    }
    pub fn strategy(
        mut self,
        v: Strategy,
    ) -> Self {
        self.strategy = Some(v);
        self
    }
    /// Set the timeout for reading stats on remote filesystems
    /// (unix only), which by default is 100ms.
    pub fn stats_timeout(
        mut self,
        v: Option<Duration>,
    ) -> Self {
        self.stats_timeout = v;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseStrategyError;
impl FromStr for Strategy {
    type Err = ParseStrategyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "iokit" => Ok(Self::Iokit),
            "diskutil" => Ok(Self::Diskutil),
            _ => Err(ParseStrategyError),
        }
    }
}
