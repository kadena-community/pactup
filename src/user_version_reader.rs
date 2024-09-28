use crate::config::PactupConfig;
use crate::user_version::UserVersion;
use crate::version_files::{get_user_version_for_directory, get_user_version_for_file};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum UserVersionReader {
  Direct(UserVersion),
  Path(PathBuf),
}

impl UserVersionReader {
  pub fn into_user_version(self, config: &PactupConfig) -> Option<UserVersion> {
    match self {
      Self::Direct(uv) => Some(uv),
      Self::Path(pathbuf) if pathbuf.is_file() => get_user_version_for_file(pathbuf, config),
      Self::Path(pathbuf) if pathbuf.is_dir() => get_user_version_for_directory(pathbuf, config),
      _ => None,
    }
  }
}

impl FromStr for UserVersionReader {
  type Err = node_semver::SemverError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let pathbuf = PathBuf::from(s);
    if pathbuf.exists() {
      Ok(Self::Path(pathbuf))
    } else {
      UserVersion::from_str(s).map(Self::Direct)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::version::Version;
  use pretty_assertions::assert_eq;
  use std::io::Write;
  use tempfile::{NamedTempFile, TempDir};

  #[test]
  fn test_file_pathbuf_to_version() {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "4").unwrap();
    let pathbuf = file.path().to_path_buf();

    let user_version = UserVersionReader::Path(pathbuf).into_user_version(&PactupConfig::default());
    assert_eq!(user_version, Some(UserVersion::OnlyMajor(4)));
  }

  #[test]
  fn test_directory_pathbuf_to_version() {
    let directory = TempDir::new().unwrap();
    let pact_version_path = directory.path().join(".pact-version");
    std::fs::write(pact_version_path, "4").unwrap();
    let pathbuf = directory.path().to_path_buf();

    let user_version = UserVersionReader::Path(pathbuf).into_user_version(&PactupConfig::default());
    assert_eq!(user_version, Some(UserVersion::OnlyMajor(4)));
  }

  #[test]
  fn test_direct_to_version() {
    let user_version = UserVersionReader::Direct(UserVersion::OnlyMajor(4))
      .into_user_version(&PactupConfig::default());
    assert_eq!(user_version, Some(UserVersion::OnlyMajor(4)));
  }

  #[test]
  fn test_from_str_directory() {
    let directory = TempDir::new().unwrap();
    let pact_version_path = directory.path().join(".pact-version");
    std::fs::write(pact_version_path, "4").unwrap();
    let pathbuf = directory.path().to_path_buf();

    let user_version = UserVersionReader::from_str(pathbuf.to_str().unwrap());
    assert!(matches!(user_version, Ok(UserVersionReader::Path(_))));
  }

  #[test]
  fn test_from_str_file() {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(b"14").unwrap();
    let pathbuf = file.path().to_path_buf();

    let user_version = UserVersionReader::from_str(pathbuf.to_str().unwrap());
    assert!(matches!(user_version, Ok(UserVersionReader::Path(_))));
  }

  #[test]
  fn test_non_existing_path() {
    let user_version = UserVersionReader::from_str("/tmp/non_existing_path");
    assert!(matches!(
      user_version,
      Ok(UserVersionReader::Direct(UserVersion::Full(
        Version::Alias(_)
      )))
    ));
  }
}
