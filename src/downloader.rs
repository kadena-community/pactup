use crate::arch::Arch;
use crate::archive;
use crate::archive::extract::ArchiveType;
use crate::archive::{Error as ExtractError, Extract};
use crate::directory_portal::DirectoryPortal;
use crate::progress::ResponseProgress;
use crate::version::Version;
use indicatif::ProgressDrawTarget;
use log::debug;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  HttpError {
    #[from]
    source: crate::http::Error,
  },
  #[error(transparent)]
  IoError {
    #[from]
    source: std::io::Error,
  },
  #[error("Can't extract the file: {}", source)]
  CantExtractFile {
    #[from]
    source: ExtractError,
  },
  // #[error("The downloaded archive is empty")]
  // TarIsEmpty,
  #[error("{} for {} not found upstream.\nYou can `pactup ls-remote` to see available versions or try a different `--arch`.", version, arch)]
  VersionNotFound { version: Version, arch: Arch },
  #[error("Version already installed at {:?}", path)]
  VersionAlreadyInstalled { path: PathBuf },
}

fn extract_archive_into(
  path: impl AsRef<Path>,
  response: impl Read,
  download_url: &Url,
) -> Result<(), Error> {
  let _ = match ArchiveType::from(download_url)? {
    ArchiveType::TarGz => archive::TarGz::new(response).extract_into(path),
    ArchiveType::TarXz => archive::TarXz::new(response).extract_into(path),
    ArchiveType::Zip => archive::Zip::new(response).extract_into(path),
  };
  Ok(())
}

/// Install a pact asset from a URL into a directory
pub fn install_pact_dist<P: AsRef<Path>>(
  version: &Version,
  download_url: &Url,
  installations_dir: P,
  arch: &Arch,
  show_progress: bool,
) -> Result<(), Error> {
  let installation_dir = PathBuf::from(installations_dir.as_ref()).join(version.v_str());

  if installation_dir.exists() {
    return Err(Error::VersionAlreadyInstalled {
      path: installation_dir,
    });
  }

  std::fs::create_dir_all(installations_dir.as_ref())?;

  let temp_installations_dir = installations_dir.as_ref().join(".downloads");
  std::fs::create_dir_all(&temp_installations_dir)?;

  let portal = DirectoryPortal::new_in(&temp_installations_dir, installation_dir);

  debug!("Going to call for {}", download_url);
  let response = crate::http::get(download_url.as_str())?;

  if response.status() == 404 {
    return Err(Error::VersionNotFound {
      version: version.clone(),
      arch: arch.clone(),
    });
  }

  debug!("Extracting response...");
  if show_progress {
    extract_archive_into(
      &portal,
      ResponseProgress::new(response, ProgressDrawTarget::stderr()),
      download_url,
    )?;
  } else {
    extract_archive_into(&portal, response, download_url)?;
  }
  // extract_archive_into(&portal, response, download_url)?;
  debug!("Extraction completed");

  // let installed_directory = std::fs::read_dir(&portal)?
  //   .next()
  //   .ok_or(Error::TarIsEmpty)??;
  // let installed_directory = installed_directory.path();

  // let renamed_installation_dir = portal.join("installation");
  // std::fs::rename(installed_directory, renamed_installation_dir)?;

  portal.teleport()?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::downloader::install_pact_dist;
  use crate::version::Version;
  use pretty_assertions::assert_eq;
  use tempfile::tempdir;

  #[test_log::test]
  fn test_installing_pact_4_11() {
    let installations_dir = tempdir().unwrap();
    let pact_path = install_in(installations_dir.path()).join("pact");
    let stdout = duct::cmd(pact_path.to_str().unwrap(), vec!["--version"])
      .stdout_capture()
      .run()
      .expect("Can't run Pact binary")
      .stdout;

    let result = String::from_utf8(stdout).expect("Can't read `pact --version` output");

    assert_eq!(result.trim(), "pact version 4.11");
  }

  fn install_in(path: &Path) -> PathBuf {
    let version = Version::parse("4.11.0").unwrap();
    let arch = Arch::X64;
    // github release asset url
    let pact_dist_mirror = Url::parse(
      "https://github.com/kadena-io/pact/releases/download/v4.11.0/pact-4.11.0-linux-20.04.zip",
    )
    .unwrap();
    install_pact_dist(&version, &pact_dist_mirror, path, &arch, false)
      .expect("Can't install Pact 4.11.0");

    let mut location_path = path.join(version.v_str());
    // .join("installation");

    if cfg!(unix) {
      location_path.push("bin");
    }
    location_path
  }
}
