use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceId(String);

impl Default for SourceId {
    fn default() -> Self {
        SourceId("".to_string())
    }
}

impl fmt::Debug for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl SourceId {
    pub fn empty() -> Self {
        SourceId::default()
    }

    pub fn repl() -> Self {
        SourceId("repl".to_string())
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        SourceId(
            path.as_ref()
                .iter()
                .map(|c| c.to_string_lossy().into_owned())
                .collect(),
        )
    }

    pub fn into_path(self) -> PathBuf {
        PathBuf::from(self.0)
    }
}

impl FromStr for SourceId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SourceId(s.to_string()))
    }
}
