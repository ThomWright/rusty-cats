use std::convert::TryFrom;
use std::path::{Path, PathBuf, StripPrefixError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalPath {
    path: PathBuf,
}
impl CanonicalPath {
    pub fn new<P>(path: P) -> Result<CanonicalPath, CanonicalPathError>
    where
        P: AsRef<Path>,
    {
        Ok(CanonicalPath {
            path: path.as_ref().canonicalize()?,
        })
    }

    pub fn parent(&self) -> Option<CanonicalPath> {
        self.path.parent().map(|p| {
            CanonicalPath::new(p).expect("parent should be canonicalisable")
        })
    }

    pub fn strip_prefix<P>(&self, base: P) -> Result<&Path, CanonicalPathError>
    where
        P: AsRef<Path>,
    {
        Ok(self.path.strip_prefix(base)?)
    }

    pub fn resolve_ts_file<P: AsRef<Path>>(
        &self,
        rel_path: P,
    ) -> Result<CanonicalPath, CanonicalPathError> {
        let mut p = self.path.join(rel_path);
        if p.is_dir() {
            p = p.join("index");
        }
        if p.extension().is_none() {
            p.set_extension("ts");
        }
        Ok(CanonicalPath::new(p)?)
    }
}

use std::fmt;
impl fmt::Display for CanonicalPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

use std::ops::Deref;
impl Deref for CanonicalPath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl AsRef<Path> for CanonicalPath {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
impl AsRef<PathBuf> for CanonicalPath {
    fn as_ref(&self) -> &PathBuf {
        &self.path
    }
}

impl TryFrom<&Path> for CanonicalPath {
    type Error = CanonicalPathError;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        Ok(CanonicalPath {
            path: path.canonicalize()?,
        })
    }
}
impl TryFrom<&PathBuf> for CanonicalPath {
    type Error = CanonicalPathError;
    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        Ok(CanonicalPath {
            path: path.canonicalize()?,
        })
    }
}

#[derive(Debug)]
pub enum CanonicalPathError {
    Io(std::io::Error),
    StripPrefix(StripPrefixError),
}

impl From<std::io::Error> for CanonicalPathError {
    fn from(e: std::io::Error) -> Self {
        CanonicalPathError::Io(e)
    }
}

impl From<StripPrefixError> for CanonicalPathError {
    fn from(e: StripPrefixError) -> Self {
        CanonicalPathError::StripPrefix(e)
    }
}
