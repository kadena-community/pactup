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
  fn extract_into<P: AsRef<Path>>(self, path: P) -> Result<(), Error>;
}

pub enum ArchiveType {
  TarGz,
  Zip,
}

impl ArchiveType {
  pub fn from(url: &url::Url) -> Result<Self, Error> {
    let archive_type = url
      .path_segments()
      .and_then(std::iter::Iterator::last)
      .and_then(|last| last.split('.').last())
      .ok_or(Error::UnknownArchiveType {
        content_type: "unknown".to_string(),
      })?;
    match archive_type {
      "gz" => Ok(Self::TarGz),
      "zip" => Ok(Self::Zip),
      _ => Err(Error::UnknownArchiveType {
        content_type: archive_type.to_string(),
      }),
    }
  }
}
