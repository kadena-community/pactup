use std::error::Error as StdError;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
  IoError(std::io::Error),
  ZipError(zip::result::ZipError),
  HttpError(crate::http::Error),
  UnknownArchiveType { content_type: String },
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::IoError(x) => x.fmt(f),
      Self::ZipError(x) => x.fmt(f),
      Self::HttpError(x) => x.fmt(f),
      Self::UnknownArchiveType { content_type } => {
        write!(f, "Unknown archive type: {content_type}")
      }
    }
  }
}

impl StdError for Error {}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    Self::IoError(err)
  }
}

impl From<zip::result::ZipError> for Error {
  fn from(err: zip::result::ZipError) -> Self {
    Self::ZipError(err)
  }
}

impl From<crate::http::Error> for Error {
  fn from(err: crate::http::Error) -> Self {
    Self::HttpError(err)
  }
}

impl From<walkdir::Error> for Error {
  fn from(err: walkdir::Error) -> Self {
    Self::IoError(err.into())
  }
}
pub trait Extract {
  fn extract_into(self: Box<Self>, path: &Path) -> Result<(), Error>;
}
