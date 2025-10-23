use serde::{Deserialize, Serialize};
use std::{
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};
use url::Url;

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SourceId {
    #[default]
    Empty,
    Repl,
    Path(Vec<String>),
    Url(Url),
}

impl fmt::Debug for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SourceId::Empty => write!(f, "?"),
            SourceId::Repl => write!(f, "repl"),
            SourceId::Path(path) => path.fmt(f),
            SourceId::Url(url) => url.fmt(f),
        }
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SourceId::Empty => write!(f, "?"),
            SourceId::Repl => write!(f, "repl"),
            SourceId::Path(path) => path.join("/").fmt(f),
            SourceId::Url(url) => url.fmt(f),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to convert SourceId to path: {source_id}")]
pub struct SourceIdToPathError {
    source_id: SourceId,
}

impl SourceId {
    pub fn empty() -> Self {
        SourceId::Empty
    }

    pub fn repl() -> Self {
        SourceId::Repl
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        SourceId::Path(
            path.as_ref()
                .iter()
                .map(|c| c.to_string_lossy().into_owned())
                .collect(),
        )
    }

    pub fn into_path(self) -> Result<PathBuf, SourceIdToPathError> {
        let SourceId::Path(path) = self else {
            return Err(SourceIdToPathError {
                source_id: self.clone(),
            });
        };
        Ok(path.iter().map(|e| e.to_string()).collect())
    }
}

impl FromStr for SourceId {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SourceId::Path(vec![s.to_string()]))
    }
}
