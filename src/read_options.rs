use std::str::FromStr;

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
    pub remote_stats: bool,
    pub strategy: Option<Strategy>,
}
impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            remote_stats: true,
            strategy: None,
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
