use super::command::Command;
use crate::alias::create_alias;
use crate::config::PactupConfig;
use crate::downloader::{install_pact_dist, Error as DownloaderError};
use crate::outln;
use crate::progress::ProgressConfig;
use crate::remote_pact_index::{self, Release};
use crate::user_version::UserVersion;
use crate::version::Version;
use crate::version_files::get_user_version_for_directory;
use colored::Colorize;
use log::debug;
use thiserror::Error;

#[derive(clap::Parser, Debug, Default)]
pub struct Install {
  /// A version string. Can be a partial semver or a 'development' version.
  pub version: Option<UserVersion>,

  /// Install latest nightly version.
  #[clap(long, conflicts_with_all = &["version", "latest"])]
  pub nightly: bool,

  /// Install latest version.
  #[clap(long, conflicts_with_all = &["version", "nightly"])]
  pub latest: bool,

  /// Show an interactive progress bar for the download status.
  #[clap(long, default_value_t)]
  #[arg(value_enum)]
  pub progress: ProgressConfig,

  /// Force install even if the version is already installed.
  #[clap(long)]
  pub force: bool,
}

impl Install {
  fn resolve_version(&self) -> Result<Option<UserVersion>, Error> {
    match (self.version.as_ref(), self.nightly, self.latest) {
      (Some(v), false, false) => Ok(Some(v.clone())),
      (None, true, false) => Ok(Some(UserVersion::Full(Version::Nightly(
        "nightly".to_string(),
      )))),
      (None, false, true) => Ok(Some(UserVersion::Full(Version::Latest))),
      (None, false, false) => Ok(None),
      _ => Err(Error::TooManyVersionsProvided),
    }
  }

  fn resolve_release(
    current_version: &UserVersion,
    config: &PactupConfig,
  ) -> Result<Release, Error> {
    match current_version {
      UserVersion::Full(Version::Semver(actual_version)) => {
        Self::resolve_semver_release(actual_version, current_version, config)
      }
      UserVersion::Full(v @ (Version::Bypassed | Version::Alias(_))) => {
        Err(Error::UninstallableVersion { version: v.clone() })
      }
      UserVersion::Full(Version::Nightly(tag)) => Self::resolve_nightly_release(tag, config),
      UserVersion::Full(Version::Latest) => Self::resolve_latest_release(config),
      _ => Self::resolve_generic_release(current_version, config),
    }
  }

  fn resolve_semver_release(
    actual_version: &node_semver::Version,
    current_version: &UserVersion,
    config: &PactupConfig,
  ) -> Result<Release, Error> {
    let available_releases = Self::get_available_releases(config)?;
    let release = current_version
      .to_release(&available_releases, config)
      .ok_or_else(|| Error::CantFindPactVersion {
        requested_version: current_version.clone(),
      })?;

    debug!(
      "Resolved {} into Pact version {}",
      Version::Semver(actual_version.clone()).v_str().cyan(),
      release.tag_name.v_str().cyan()
    );

    Ok(release.clone())
  }

  fn resolve_nightly_release(nightly_tag: &str, config: &PactupConfig) -> Result<Release, Error> {
    let release = remote_pact_index::get_by_tag(config.repo_urls(), &nightly_tag.to_string())
      .map_err(|_| Error::CantFindNightly {
        nightly_tag: nightly_tag.to_string(),
      })?;

    debug!(
      "Resolved nightly into Pact version {}",
      release.tag_name.v_str().cyan()
    );

    Ok(release)
  }

  fn resolve_latest_release(config: &PactupConfig) -> Result<Release, Error> {
    let release =
      remote_pact_index::latest(config.repo_urls()).map_err(|_| Error::CantFindLatest)?;

    debug!(
      "Resolved latest into Pact version {}",
      release.tag_name.v_str().cyan()
    );

    Ok(release)
  }

  fn resolve_generic_release(
    current_version: &UserVersion,
    config: &PactupConfig,
  ) -> Result<Release, Error> {
    let available_releases = Self::get_available_releases(config)?;
    current_version
      .to_release(&available_releases, config)
      .ok_or_else(|| Error::CantFindPactVersion {
        requested_version: current_version.clone(),
      })
      .cloned()
  }

  fn get_available_releases(config: &PactupConfig) -> Result<Vec<Release>, Error> {
    remote_pact_index::list(config.repo_urls())
      .map_err(|source| Error::CantListRemoteVersions { source })
  }

  fn handle_installation(
    &self,
    release: &Release,
    current_version: &UserVersion,
    config: &PactupConfig,
  ) -> Result<(), Error> {
    let version = &release.tag_name;
    outln!(
      config,
      Info,
      "Installing {} ({})",
      format!("Pact {version}",).cyan(),
      config.arch.as_str()
    );

    let download_url = release
      .download_url()
      .ok_or_else(|| Error::CantFindReleaseAsset {
        requested_version: current_version.clone(),
      })?;

    self.perform_installation(version, &download_url, config)?;
    Self::handle_aliases(release, current_version, config)?;

    Ok(())
  }

  fn perform_installation(
    &self,
    version: &Version,
    download_url: &url::Url,
    config: &PactupConfig,
  ) -> Result<(), Error> {
    match install_pact_dist(
      version,
      download_url,
      config.installations_dir(),
      config.arch,
      self.progress.enabled(config),
      self.force,
    ) {
      Err(err @ DownloaderError::VersionAlreadyInstalled { .. }) => {
        outln!(config, Error, "{} {}", "warning:".bold().yellow(), err);
        Ok(())
      }
      Err(source) => Err(Error::DownloadError { source }),
      Ok(()) => Ok(()),
    }
  }

  fn handle_aliases(
    release: &Release,
    current_version: &UserVersion,
    config: &PactupConfig,
  ) -> Result<(), Error> {
    if !config.default_version_dir().exists() {
      debug!(
        "Tagging {} as the default version",
        release.tag_name.v_str().cyan()
      );
      create_alias(config, "default", &release.tag_name)?;
    }

    if let Some(tagged_alias) = current_version.inferred_alias() {
      tag_alias(config, &release.tag_name, &tagged_alias)?;
    }

    Ok(())
  }
}

impl Command for Install {
  type Error = Error;

  fn apply(self, config: &PactupConfig) -> Result<(), Self::Error> {
    let current_dir = std::env::current_dir()?;
    let current_version = self
      .resolve_version()?
      .or_else(|| get_user_version_for_directory(&current_dir, config))
      .ok_or(Error::CantInferVersion)?;

    let release = Self::resolve_release(&current_version, config)?;
    self.handle_installation(&release, &current_version, config)
  }
}

fn tag_alias(
  config: &PactupConfig,
  matched_version: &Version,
  alias: &Version,
) -> Result<(), Error> {
  let alias_name = alias.v_str();
  debug!(
    "Tagging {} as alias for {}",
    alias_name.cyan(),
    matched_version.v_str().cyan()
  );
  create_alias(config, &alias_name, matched_version)?;
  Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
  #[error("Can't download the requested binary: {source}")]
  DownloadError { source: DownloaderError },
  #[error(transparent)]
  IoError {
    #[from]
    source: std::io::Error,
  },
  #[error("Can't find version in dotfiles. Please provide a version manually to the command.")]
  CantInferVersion,
  #[error(transparent)]
  CantListRemoteVersions { source: remote_pact_index::Error },
  #[error("Can't find a Pact version that matches {requested_version} in remote")]
  CantFindPactVersion { requested_version: UserVersion },
  #[error("Can't find a release asset for the requested version: {requested_version}")]
  CantFindReleaseAsset { requested_version: UserVersion },
  #[error("Can't find nightly version named {nightly_tag}")]
  CantFindNightly { nightly_tag: String },
  #[error("Can't find any versions in the upstream version index.")]
  CantFindLatest,
  #[error("The requested version is not installable: {}", version.v_str())]
  UninstallableVersion { version: Version },
  #[error("Too many versions provided. Please don't use --nightly with a version string.")]
  TooManyVersionsProvided,
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use std::str::FromStr;

  fn create_test_config() -> PactupConfig {
    let base_dir = tempfile::tempdir().unwrap();
    PactupConfig::default().with_base_dir(Some(base_dir.path().to_path_buf()))
  }

  #[test]
  fn test_resolve_version() {
    let test_cases = vec![
      (
        Install {
          version: Some(UserVersion::from_str("4.13.0").unwrap()),
          nightly: false,
          latest: false,
          force: false,
          progress: ProgressConfig::Never,
        },
        Ok(Some(UserVersion::from_str("4.13.0").unwrap())),
      ),
      (
        Install {
          version: None,
          nightly: true,
          latest: false,
          force: false,
          progress: ProgressConfig::Never,
        },
        Ok(Some(UserVersion::Full(Version::Nightly(
          "nightly".to_string(),
        )))),
      ),
      (
        Install {
          version: None,
          nightly: false,
          latest: true,
          force: false,
          progress: ProgressConfig::Never,
        },
        Ok(Some(UserVersion::Full(Version::Latest))),
      ),
      (
        Install {
          version: Some(UserVersion::from_str("4.13.0").unwrap()),
          nightly: true,
          latest: false,
          force: false,
          progress: ProgressConfig::Never,
        },
        Err(Error::TooManyVersionsProvided),
      ),
    ];

    for (install, expected) in test_cases {
      assert_eq!(
        format!("{:?}", install.resolve_version()),
        format!("{:?}", expected)
      );
    }
  }

  #[test]
  #[cfg(target_os = "linux")]
  fn test_set_default_on_new_installation() {
    let config = create_test_config();
    assert!(!config.default_version_dir().exists());

    Install {
      version: UserVersion::from_str("4.13.0").ok(),
      nightly: false,
      latest: false,
      force: false,
      progress: ProgressConfig::Never,
    }
    .apply(&config)
    .expect("Can't install");

    assert!(config.default_version_dir().exists());
    assert_eq!(
      config.default_version_dir().canonicalize().ok(),
      config
        .installations_dir()
        .join("v4.13.0")
        .canonicalize()
        .ok()
    );
  }

  #[test]
  #[cfg(target_os = "linux")]
  fn test_install_latest() {
    let config = create_test_config();

    Install {
      version: None,
      nightly: false,
      latest: true,
      force: false,
      progress: ProgressConfig::Never,
    }
    .apply(&config)
    .expect("Can't install");

    let latest_release =
      remote_pact_index::latest(config.repo_urls()).expect("Can't get pact version list");
    let latest_version = latest_release.tag_name;
    assert!(config.installations_dir().exists());
    assert!(config
      .installations_dir()
      .join(latest_version.to_string())
      .canonicalize()
      .unwrap()
      .exists());
  }

  #[test]
  #[cfg(target_os = "linux")]
  fn test_install_nightly() {
    let config = create_test_config();

    Install {
      version: None,
      nightly: true,
      latest: false,
      force: false,
      progress: ProgressConfig::Never,
    }
    .apply(&config)
    .expect("Can't install");

    let nightly_release =
      remote_pact_index::get_by_tag(config.repo_urls(), &String::from("nightly"))
        .expect("Can't get pact version list");
    let nightly_version = nightly_release.tag_name;
    assert!(config.installations_dir().exists());
    assert!(config
      .installations_dir()
      .join(nightly_version.to_string())
      .canonicalize()
      .unwrap()
      .exists());
  }

  #[test]
  fn test_uninstallable_version() {
    let config = create_test_config();
    let result = Install {
      version: Some(UserVersion::Full(Version::Bypassed)),
      nightly: false,
      latest: false,
      force: false,
      progress: ProgressConfig::Never,
    }
    .apply(&config);

    assert!(matches!(result, Err(Error::UninstallableVersion { .. })));
  }

  #[test]
  fn test_too_many_versions() {
    let config = create_test_config();
    let result = Install {
      version: Some(UserVersion::from_str("4.13.0").unwrap()),
      nightly: true,
      latest: false,
      force: false,
      progress: ProgressConfig::Never,
    }
    .apply(&config);

    assert!(matches!(result, Err(Error::TooManyVersionsProvided)));
  }
}
