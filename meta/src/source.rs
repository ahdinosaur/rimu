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

impl SourceId {
    pub fn empty() -> Self {
        SourceId::Empty
    }

    pub fn repl() -> Self {
        SourceId::Repl
    }

    #[allow(clippy::result_unit_err)]
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, ()> {
        Ok(SourceId::Url(Url::from_file_path(path)?))
    }

    #[allow(clippy::result_unit_err)]
    pub fn to_path(&self) -> Result<PathBuf, ()> {
        let SourceId::Url(url) = self else {
            return Err(());
        };
        url.to_file_path()
    }
}

impl FromStr for SourceId {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SourceId::Url(Url::parse(s)?))
    }
}
