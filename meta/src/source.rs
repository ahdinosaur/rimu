use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct SourceId(Vec<String>);

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "?")
        } else {
            write!(f, "{}", self.0.clone().join("/"))
        }
    }
}

impl fmt::Debug for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl SourceId {
    pub fn empty() -> Self {
        SourceId(Vec::new())
    }

    pub fn repl() -> Self {
        SourceId(vec!["repl".to_string()])
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        SourceId(
            path.as_ref()
                .iter()
                .map(|c| c.to_string_lossy().into_owned())
                .collect(),
        )
    }

    pub fn to_path(&self) -> PathBuf {
        self.0.iter().map(|e| e.to_string()).collect()
    }
}

impl FromStr for SourceId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SourceId(vec![s.to_string()]))
    }
}
