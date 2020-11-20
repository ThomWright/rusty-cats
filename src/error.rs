use crate::canonical_path::CanonicalPathError;

#[derive(Debug)]
pub enum CatsError {
  Io(std::io::Error),
  CanonicalPath(CanonicalPathError),
}

impl From<std::io::Error> for CatsError {
  fn from(e: std::io::Error) -> Self {
    CatsError::Io(e)
  }
}

impl From<CanonicalPathError> for CatsError {
  fn from(e: CanonicalPathError) -> Self {
    CatsError::CanonicalPath(e)
  }
}
