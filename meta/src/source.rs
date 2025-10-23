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

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SourceId {
    fn from(value: String) -> Self {
        SourceId(value)
    }
}

impl From<SourceId> for String {
    fn from(value: SourceId) -> Self {
        value.0
    }
}

impl From<PathBuf> for SourceId {
    fn from(value: PathBuf) -> Self {
        SourceId::from_path(value)
    }
}

impl FromStr for SourceId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SourceId(s.to_string()))
    }
}

impl AsRef<str> for SourceId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
