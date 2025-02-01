use crate::archive::{Archive, Error as ExtractError};
use crate::directory_portal::DirectoryPortal;
use crate::progress::ResponseProgress;
use crate::system_info::PlatformArch;
use crate::version::Version;
use indicatif::ProgressDrawTarget;
use log::debug;
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
  #[error("The downloaded archive is empty")]
  TarIsEmpty,
  #[error("{} for {} not found upstream.\nYou can `pactup ls-remote` to see available versions or try a different `--arch`.", version, arch)]
  VersionNotFound {
    version: Version,
    arch: PlatformArch,
  },
  #[error("Version already installed at {:?}", path)]
  VersionAlreadyInstalled { path: PathBuf },
}

/// Install a pact asset from a URL into a directory
pub fn install_pact_dist<P: AsRef<Path>>(
  version: &Version,
  download_url: &Url,
  installations_dir: P,
  arch: PlatformArch,
  show_progress: bool,
  force: bool,
) -> Result<(), Error> {
  let version_installation_dir = PathBuf::from(installations_dir.as_ref()).join(version.v_str());

  if version_installation_dir.exists() {
    if force {
      debug!("Removing directory {:?}", version_installation_dir);
      std::fs::remove_dir_all(&version_installation_dir)?;
    } else {
      return Err(Error::VersionAlreadyInstalled {
        path: version_installation_dir,
      });
    }
  }
  if !installations_dir.as_ref().exists() {
    debug!("Creating directory {:?}", installations_dir.as_ref());
    std::fs::create_dir_all(installations_dir.as_ref())?;
  }

  let temp_installations_dir = installations_dir.as_ref().join(".downloads");
  if temp_installations_dir.exists() {
    debug!("Removing directory {:?}", temp_installations_dir);
    std::fs::remove_dir_all(&temp_installations_dir)?;
  }
  std::fs::create_dir_all(&temp_installations_dir)?;

  debug!("Creating directory portal");
  let portal = DirectoryPortal::new_in(&temp_installations_dir, version_installation_dir);

  for _ in Archive::supported() {
    debug!("Going to call for {}", download_url);
    let response = crate::http::get(download_url.as_str())?;

    if !response.status().is_success() {
      continue;
    }

    debug!("Extracting response...");
    if show_progress {
      Archive::extract_archive_into(
        portal.as_ref(),
        ResponseProgress::new(response, ProgressDrawTarget::stderr()),
        download_url.as_str(),
      )?;
    } else {
      Archive::extract_archive_into(portal.as_ref(), response, download_url.as_str())?;
    }
    debug!("Extraction completed");
    std::fs::read_dir(&portal)?
      .next()
      .ok_or(Error::TarIsEmpty)??;

    portal.teleport()?;

    return Ok(());
  }

  Err(Error::VersionNotFound {
    version: version.clone(),
    arch,
  })
}

#[cfg(test)]
#[cfg(target_os = "linux")]
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

    assert_eq!(result.trim(), "pact version 4.13");
  }

  fn install_in(path: &Path) -> PathBuf {
    let version = Version::parse("4.13.0").unwrap();
    #[cfg(target_arch = "x86_64")]
    let arch = PlatformArch::X64;
    #[cfg(target_arch = "aarch64")]
    let arch = PlatformArch::Arm64;
    // github release asset url
    let pact_dist_mirror = Url::parse(
      "https://github.com/kadena-io/pact/releases/download/v4.13.0/pact-4.13.0-linux-20.04.zip",
    )
    .unwrap();
    install_pact_dist(&version, &pact_dist_mirror, path, arch, false, false)
      .expect("Can't install Pact 4.13.0");

    let mut location_path = path.join(version.v_str());
    // .join("installation");

    if cfg!(unix) {
      location_path.push("bin");
    }
    location_path
  }
}
