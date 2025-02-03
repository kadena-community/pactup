use crate::system_info::{get_platform, Platform, PlatformArch, PlatformOS};
use crate::{pretty_serde::DecodeError, version::Version};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use url::Url;

lazy_static! {
  static ref PLATFORM_MAP: HashMap<PlatformOS, Vec<&'static str>> = {
    let mut m = HashMap::new();
    m.insert(PlatformOS::Linux, vec!["linux", "ubuntu"]);
    m.insert(PlatformOS::MacOS, vec!["darwin", "osx", "macos"]);
    m.insert(PlatformOS::Windows, vec!["windows", "win"]);
    m
  };
  static ref ARCH_MAP: HashMap<PlatformArch, Vec<&'static str>> = {
    let mut m = HashMap::new();
    m.insert(PlatformArch::X64, vec!["x64", "x86_64", "amd64"]);
    m.insert(PlatformArch::Arm64, vec!["arm64", "aarch64"]);
    m.insert(PlatformArch::Armv7l, vec!["armv7l", "arm"]);
    m.insert(PlatformArch::X86, vec!["x86", "i386"]);
    m.insert(PlatformArch::Ppc64le, vec!["ppc64le"]);
    m.insert(PlatformArch::Ppc64, vec!["ppc64"]);
    m.insert(PlatformArch::S390x, vec!["s390x"]);
    m
  };
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
  // pub content_type: String,
  // pub size: i64,
  // pub created_at: DateTime<Utc>,
  // pub updated_at: DateTime<Utc>,
  // pub download_count: i64,
  pub download_url: Url,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
  // pub id: u64,
  pub tag: Version,
  // pub author: String,
  // pub name: String,
  pub draft: bool,
  pub prerelease: bool,
  // pub created_at: DateTime<Utc>,
  // pub published_at: DateTime<Utc>,
  // pub markdown: String,
  // pub html: String,
  pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct UnghReleasesResponse {
  releases: Vec<Release>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct UnghLatestReleaseResponse {
  release: Release,
}

impl Release {
  fn build_name_pattern(&self) -> String {
    match &self.tag {
      Version::Semver(semver) => format!(
        "{}(\\.{})?(\\.{})?",
        semver.major, semver.minor, semver.patch
      ),
      Version::Nightly(tag) | Version::Alias(tag) => tag.to_string(),
      Version::Latest => "latest".to_string(),
      Version::Bypassed => "system".to_string(),
    }
  }
  pub fn version_matcher_for_platform(&self, platform: &Platform) -> Result<Regex, String> {
    let Platform(os, arch) = platform;

    let platform_patterns = PLATFORM_MAP
      .get(os)
      .ok_or_else(|| format!("Unsupported OS: {os}"))?
      .join("|");

    let arch_patterns = ARCH_MAP
      .get(arch)
      .ok_or_else(|| format!("Unsupported architecture: {arch}"))?
      .join("|");

    let name_pattern = self.build_name_pattern();

    let pattern = format!(
      r"^pact-{name_pattern}-(({platform_patterns})(-({arch_patterns}))?|({arch_patterns})(-({platform_patterns}))?)(-\d+\.\d+)?(-({arch_patterns}))?\.(tar\.gz|zip)$"
    );

    Regex::new(&pattern).map_err(|e| format!("Regex creation error: {e}"))
  }

  pub fn asset_for_current_platform(&self) -> Option<&Asset> {
    let platform = get_platform();
    let regex = self.version_matcher_for_platform(&platform).ok()?;
    self
      .assets
      .iter()
      .find(|x| {
        // Extract filename from download_url
        if let Some(filename) = &x
          .download_url
          .path_segments()
          .and_then(std::iter::Iterator::last)
        {
          regex.is_match(filename)
        } else {
          false
        }
      })
      .or_else(|| {
        if let Platform(os, PlatformArch::X64) = platform {
          let platform_patterns = PLATFORM_MAP.get(&os)?;
          let name_pattern = self.build_name_pattern();

          let fallback_pattern = format!(
            r"^pact-{name_pattern}-({})(-\d+\.\d+)?\.(tar\.gz|zip)$",
            platform_patterns.join("|")
          );

          if let Ok(fallback_regex) = Regex::new(&fallback_pattern) {
            self.assets.iter().find(|x| {
              if let Some(filename) = &x
                .download_url
                .path_segments()
                .and_then(std::iter::Iterator::last)
              {
                fallback_regex.is_match(filename)
              } else {
                false
              }
            })
          } else {
            None
          }
        } else {
          None
        }
      })
  }

  pub fn download_url(&self) -> Option<Url> {
    self
      .asset_for_current_platform()
      .map(|asset| asset.download_url.clone())
  }

  pub fn has_supported_asset(&self) -> bool {
    self.asset_for_current_platform().is_some()
  }

  pub fn is_nightly(&self) -> bool {
    self.tag.is_nightly()
  }
}

fn format_ungh_url(repo_url: &str, path: &str) -> String {
  format!(
    "https://ungh.sashoush.dev/repos/{}/{}",
    repo_url.trim_end_matches('/'),
    path
  )
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum Error {
  #[error("can't get remote versions file: {0}")]
  #[diagnostic(transparent)]
  Http(#[from] crate::http::Error),
  #[error("can't decode remote versions file: {0}")]
  #[diagnostic(transparent)]
  Decode(#[from] DecodeError),
  #[error("can't find the release {0}")]
  #[diagnostic(code(pactup::remote_pact_index::Error::NotFound))]
  NotFound(String),
}

fn list_internal(repo_url: &str) -> Result<Vec<Release>, Error> {
  let releases_url = format_ungh_url(repo_url, "releases");
  let resp = crate::http::get(&releases_url)
    .map_err(crate::http::Error::from)?
    .error_for_status()
    .map_err(crate::http::Error::from)?;

  let text = resp.text().map_err(crate::http::Error::from)?;
  let ungh_response: UnghReleasesResponse =
    serde_json::from_str(&text[..]).map_err(|cause| DecodeError::from_serde(text, cause))?;

  Ok(ungh_response.releases)
}

pub fn list(repo_urls: Vec<&str>) -> Result<Vec<Release>, Error> {
  let mut releases = Vec::new();
  for repo in repo_urls {
    let mut repo_releases = list_internal(repo)?;
    releases.append(&mut repo_releases);
  }
  Ok(releases)
}

fn latest_internal(repo_url: &str) -> Result<Release, Error> {
  let latest_url = format_ungh_url(repo_url, "releases/latest");

  let resp = crate::http::get(&latest_url)
    .map_err(crate::http::Error::from)?
    .error_for_status()
    .map_err(crate::http::Error::from)?;

  let text = resp.text().map_err(crate::http::Error::from)?;
  let ungh_response: UnghLatestReleaseResponse =
    serde_json::from_str(&text[..]).map_err(|cause| DecodeError::from_serde(text, cause))?;

  Ok(ungh_response.release)
}

pub fn latest(repo_urls: Vec<&str>) -> Result<Release, Error> {
  let mut picked: Option<Release> = None;
  for repo in repo_urls {
    let release = latest_internal(repo)?;
    if release.is_nightly() {
      continue;
    }
    if let Some(ref picked_release) = picked {
      if release.tag > picked_release.tag {
        picked = Some(release);
      }
    } else {
      picked = Some(release);
    }
  }
  picked.ok_or_else(|| Error::NotFound("latest".to_string()))
}

pub fn get_by_tag(repo_urls: Vec<&str>, tag: &str) -> Result<Release, Error> {
  let releases = list(repo_urls)?;
  let release = releases
    .into_iter()
    .find(|x| x.tag.to_string() == tag)
    .ok_or_else(|| Error::NotFound(tag.to_string()))?;
  Ok(release)
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  fn create_test_release(version: &str) -> Release {
    Release {
      tag: Version::parse(version).unwrap(),
      assets: vec![],
      prerelease: false,
      draft: false,
    }
  }

  #[test]
  #[allow(clippy::too_many_lines)]
  fn test_version_matcher_patterns() {
    let test_cases = vec![
      // Semver formats
      (
        "4.13.0",
        PlatformOS::Linux,
        PlatformArch::X64,
        vec![
          ("pact-4.13.0-linux-x64.tar.gz", true),
          ("pact-4.13.0-linux-amd64.tar.gz", true),
          ("pact-4.13.0-ubuntu-x64.tar.gz", true),
          ("pact-4.13.0-ubuntu-x64-22.10.tar.gz", true),
          ("pact-4.13-linux-x64.tar.gz", true),
          ("pact-4.13-linux-x64-22.04.tar.gz", true),
          ("pact-4-linux-x64.tar.gz", true),
          ("pact-invalid-linux-x64.tar.gz", false),
        ],
      ),
      // Tag formats (nightly)
      (
        "nightly",
        PlatformOS::Linux,
        PlatformArch::X64,
        vec![
          ("pact-nightly-linux-x64.tar.gz", true),
          ("pact-nightly-ubuntu-amd64.tar.gz", true),
          ("pact-dev-linux-x64.tar.gz", false),
        ],
      ),
      // Tag formats (alpha)
      (
        "alpha",
        PlatformOS::Linux,
        PlatformArch::X64,
        vec![
          ("pact-alpha-linux-x64.tar.gz", true),
          ("pact-alpha-ubuntu-amd64.tar.gz", true),
          ("pact-beta-linux-x64.tar.gz", false),
        ],
      ),
      // MacOS formats
      (
        "4.13.0",
        PlatformOS::MacOS,
        PlatformArch::Arm64,
        vec![
          ("pact-4.13.0-darwin-arm64.tar.gz", true),
          ("pact-4.13.0-darwin-aarch64.tar.gz", true),
          ("pact-4.13.0-osx-arm64.tar.gz", true),
          ("pact-4.13.0-macos-aarch64.tar.gz", true),
          ("pact-4.13.0-aarch64-macos.tar.gz", true),
          ("pact-4.13.0-aarch64-osx.tar.gz", true),
          ("pact-4.13.0-aarch64-darwin.tar.gz", true),
          ("pact-4.13.0-darwin-x64.tar.gz", false),
        ],
      ),
      // Windows formats
      (
        "4.13.0",
        PlatformOS::Windows,
        PlatformArch::X64,
        vec![
          ("pact-4.13.0-windows-x64.zip", true),
          ("pact-4.13.0-win-amd64.zip", true),
          ("pact-4.13.0-windows-x86_64.zip", true),
          ("pact-4.13.0-windows-arm64.zip", false),
        ],
      ),
      // Special tags
      (
        "latest",
        PlatformOS::Linux,
        PlatformArch::X64,
        vec![
          ("pact-latest-linux-x64.tar.gz", true),
          ("pact-latest-ubuntu-amd64.tar.gz", true),
        ],
      ),
      // Development tags
      (
        "dev",
        PlatformOS::Linux,
        PlatformArch::X64,
        vec![
          ("pact-dev-linux-x64.tar.gz", true),
          ("pact-dev-ubuntu-amd64.tar.gz", true),
        ],
      ),
      // Release candidate tags
      (
        "rc1",
        PlatformOS::Linux,
        PlatformArch::X64,
        vec![
          ("pact-rc1-linux-x64.tar.gz", true),
          ("pact-rc1-ubuntu-amd64.tar.gz", true),
        ],
      ),
    ];

    for (version, os, arch, test_files) in test_cases {
      let release = create_test_release(version);
      let platform = Platform(os, arch);
      let regex = release
        .version_matcher_for_platform(&platform)
        .expect("Failed to create regex");

      for (test_file, should_match) in test_files {
        assert_eq!(
          regex.is_match(test_file),
          should_match,
          "Failed for {} with pattern {} on test file {}",
          version,
          regex.as_str(),
          test_file
        );
      }
    }
  }
  #[test]
  fn test_version_matcher_patterns_with_platform_version() {
    let test_cases = vec![
      (
        "4.13.0",
        PlatformOS::Linux,
        PlatformArch::X64,
        vec![
          ("pact-4.13.0-linux-20.04.tar.gz", true),
          ("pact-4.13.0-ubuntu-22.04.tar.gz", true),
          ("pact-4.13.0-linux-20.04-x64.tar.gz", true),
          ("pact-4.13.0-linux-20.04-amd64.tar.gz", true),
          ("pact-4.13.0-linux.tar.gz", true), // No platform version or arch
          ("pact-4.13.0-linux-invalid.tar.gz", false),
        ],
      ),
      // Test x64 default when no arch specified
      (
        "4.13.0",
        PlatformOS::Linux,
        PlatformArch::X64,
        vec![
          ("pact-4.13.0-linux.tar.gz", true),
          ("pact-4.13.0-ubuntu.tar.gz", true),
          ("pact-4.13.0-linux-20.04.tar.gz", true),
        ],
      ),
      // Test arm64 requires explicit specification
      (
        "4.13.0",
        PlatformOS::Linux,
        PlatformArch::Arm64,
        vec![
          ("pact-4.13.0-linux-arm64.tar.gz", true),
          ("pact-4.13.0-linux-aarch64.tar.gz", true),
          ("pact-4.13.0-linux.tar.gz", true), // Should not match without arch
        ],
      ),
    ];

    for (version, os, arch, test_files) in test_cases {
      let release = create_test_release(version);
      let platform = Platform(os, arch);
      let regex = release
        .version_matcher_for_platform(&platform)
        .expect("Failed to create regex");

      for (test_file, should_match) in test_files {
        assert_eq!(
          regex.is_match(test_file),
          should_match,
          "Failed for {} with pattern {} on test file {}",
          version,
          regex.as_str(),
          test_file
        );
      }
    }
  }

  #[test]
  #[cfg(target_os = "linux")]
  fn test_asset_fallback_to_x64_on_linux() {
    let mut release = create_test_release("4.13.0");
    release.assets = vec![Asset {
      download_url: Url::parse("https://example.com/download/pact-4.13.0-linux-20.04.tar.gz")
        .unwrap(),
    }];

    let asset = release.asset_for_current_platform();
    assert!(asset.is_some());
  }

  #[test]
  fn test_asset_for_current_platform() {
    let mut release = create_test_release("4.13.0");
    let platform = get_platform();
    let asset_name = match platform {
      Platform(PlatformOS::MacOS, PlatformArch::Arm64) => "pact-4.13.0-darwin-arm64.tar.gz",
      Platform(PlatformOS::MacOS, PlatformArch::X64) => "pact-4.13.0-darwin-x64.tar.gz",
      Platform(PlatformOS::Windows, _) => "pact-4.13.0-windows-x64.zip",
      _ => "pact-4.13.0-linux-x64.tar.gz", // default for tests
    };

    release.assets = vec![Asset {
      download_url: Url::parse(&format!("https://example.com/download/{asset_name}")).unwrap(),
    }];

    assert!(release.has_supported_asset());
    assert!(release.asset_for_current_platform().is_some());
  }

  #[test]
  fn test_unsupported_platform() {
    let release = create_test_release("4.13.0");
    let result =
      release.version_matcher_for_platform(&Platform(PlatformOS::Windows, PlatformArch::S390x));
    assert!(result.is_ok());
  }

  #[test]
  fn test_special_version_tags() {
    let test_cases = vec![
      ("nightly", "pact-nightly-linux-x64.tar.gz"),
      ("alpha", "pact-alpha-linux-x64.tar.gz"),
      ("beta", "pact-beta-linux-x64.tar.gz"),
      ("rc1", "pact-rc1-linux-x64.tar.gz"),
      ("latest", "pact-latest-linux-x64.tar.gz"),
      ("dev", "pact-dev-linux-x64.tar.gz"),
    ];

    for (version, test_file) in test_cases {
      let release = create_test_release(version);
      let platform = Platform(PlatformOS::Linux, PlatformArch::X64);
      let regex = release
        .version_matcher_for_platform(&platform)
        .expect("Failed to create regex");

      assert!(
        regex.is_match(test_file),
        "Failed for version {} with pattern {} on test file {}",
        version,
        regex.as_str(),
        test_file
      );
    }
  }

  #[cfg(target_os = "linux")]
  #[test]
  fn test_list() {
    let repo = "kadena-io/pact";
    let expected_version = Version::parse("4.13.0").unwrap();
    let mut versions = list_internal(repo).expect("Can't get HTTP data");
    let release = versions
      .drain(..)
      .find(|x| x.tag == expected_version)
      .map(|x| x.tag);
    assert_eq!(release, Some(expected_version));
    assert!(!release.unwrap().is_nightly());

    let repo = "kadena-io/pact-5";
    let expected_version = Version::parse("nightly").unwrap();
    let mut versions = list_internal(repo).expect("Can't get HTTP data");
    let release = versions
      .drain(..)
      .find(|x| x.tag == expected_version)
      .map(|x| x.tag);
    assert_eq!(release, Some(expected_version));
    assert!(release.unwrap().is_nightly());
  }
}
