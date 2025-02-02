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
