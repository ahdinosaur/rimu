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
    Url(Url),
}

impl fmt::Debug for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SourceId::Empty => write!(f, "?"),
            SourceId::Repl => write!(f, "repl"),
            SourceId::Url(url) => url.fmt(f),
        }
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SourceId::Empty => write!(f, "?"),
            SourceId::Repl => write!(f, "repl"),
            SourceId::Url(url) => url.fmt(f),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to create SourceId from path: {path}")]
pub struct SourceIdFromPathError {
    path: PathBuf,
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

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, SourceIdFromPathError> {
        Ok(SourceId::Url(Url::from_file_path(&path).map_err(|_| {
            SourceIdFromPathError {
                path: path.as_ref().to_path_buf(),
            }
        })?))
    }

    pub fn to_path(&self) -> Result<PathBuf, SourceIdToPathError> {
        let SourceId::Url(url) = self else {
            return Err(SourceIdToPathError {
                source_id: self.clone(),
            });
        };
        url.to_file_path().map_err(|_| SourceIdToPathError {
            source_id: self.clone(),
        })
    }
}

impl FromStr for SourceId {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SourceId::Url(Url::parse(s)?))
    }
}
