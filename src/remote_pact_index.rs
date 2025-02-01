use std::collections::HashMap;
use std::fmt::Debug;

use crate::system_info::{get_platform, Platform, PlatformArch, PlatformOS};
use crate::{pretty_serde::DecodeError, version::Version};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Asset {
  pub url: Url,
  pub browser_download_url: Url,
  pub id: usize,
  pub name: String,
  pub size: i64,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// The Release struct holds release information from the repository.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Release {
  pub tag_name: Version,
  pub assets: Vec<Asset>,
  pub prerelease: bool,
  pub draft: bool,
}

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

impl Release {
  fn build_name_pattern(&self) -> String {
    match &self.tag_name {
      Version::Semver(semver) => format!(
        "{}(\\.{})?(\\.{})?",
        semver.major, semver.minor, semver.patch
      ),
      Version::Nightly(tag) | Version::Alias(tag) => tag.to_string(),
      Version::Latest => "latest".to_string(),
      Version::Bypassed => "system".to_string(),
    }
  }

  /// Returns the appropriate version matcher for the specified platform.
  pub fn version_matcher_for_platform(&self, platform: &Platform) -> Result<Regex, String> {
    let Platform(os, arch) = platform;

    // Get platform aliases
    let platform_patterns = PLATFORM_MAP
      .get(os)
      .ok_or_else(|| format!("Unsupported OS: {os}"))?
      .join("|");

    // Get architecture aliases
    let arch_patterns = ARCH_MAP
      .get(arch)
      .ok_or_else(|| format!("Unsupported architecture: {arch}"))?
      .join("|");

    let name_pattern = self.build_name_pattern();

    // Build the complete regex pattern
    // Now includes optional platform version (e.g., -20.04) and optional architecture
    let pattern = format!(
      r"^pact-{name_pattern}-({platform_patterns})(-\d+\.\d+)?(-({arch_patterns}))?\.(tar\.gz|zip)$"
    );

    Regex::new(&pattern).map_err(|e| format!("Regex creation error: {e}"))
  }

  /// Finds the asset for the current architecture and platform.
  pub fn asset_for_current_platform(&self) -> Option<&Asset> {
    let platform = get_platform();
    let regex = self.version_matcher_for_platform(&platform).ok()?;

    // First try to find an asset with explicit architecture
    self
      .assets
      .iter()
      .find(|x| regex.is_match(&x.name))
      .or_else(|| {
        // If no explicit architecture found and current arch is x64,
        // try to find asset without architecture specification
        if let Platform(os, PlatformArch::X64) = platform {
          let platform_patterns = PLATFORM_MAP.get(&os)?;
          let name_pattern = self.build_name_pattern();

          // Pattern for assets without architecture specification
          let fallback_pattern = format!(
            r"^pact-{name_pattern}-({})(-\d+\.\d+)?\.(tar\.gz|zip)$",
            platform_patterns.join("|")
          );

          if let Ok(fallback_regex) = Regex::new(&fallback_pattern) {
            self
              .assets
              .iter()
              .find(|x| fallback_regex.is_match(&x.name))
          } else {
            None
          }
        } else {
          None
        }
      })
  }

  /// Checks if the release has a supported asset for the current platform.
  pub fn has_supported_asset(&self) -> bool {
    self.asset_for_current_platform().is_some()
  }

  pub fn download_url(&self) -> Option<Url> {
    self
      .asset_for_current_platform()
      .map(|asset| asset.browser_download_url.clone())
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct GitHubRateLimitError {
  message: String,
  documentation_url: String,
}

fn handle_github_rate_limit(resp: reqwest::blocking::Response) -> reqwest::blocking::Response {
  if resp.status().as_u16() == 403 {
    let reset_time = resp
      .headers()
      .get("X-RateLimit-Reset")
      .expect("Can't get X-RateLimit-Reset header")
      .to_str()
      .expect("Can't convert X-RateLimit-Reset header to string")
      .parse::<i64>()
      .expect("Can't parse X-RateLimit-Reset header");
    let reset_time = DateTime::from_timestamp(reset_time, 0).unwrap();
    println!("GitHub rate limit exceeded. Please wait until {reset_time} to try again.");
  }
  resp
}

fn format_url(repo_url: &str, path: &str) -> String {
  format!("https://api.github.com/repos/{repo_url}/{path}")
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum Error {
  #[error("can't get remote versions file: {0}")]
  #[diagnostic(transparent)]
  Http(#[from] crate::http::Error),
  #[error("can't decode remote versions file: {0}")]
  #[diagnostic(transparent)]
  Decode(#[from] DecodeError),
}

fn list_internal(repo_url: &str) -> Result<Vec<Release>, Error> {
  let base_url = repo_url.trim_end_matches('/');
  let index_json_url = format_url(base_url, "releases");
  let resp = crate::http::get(&index_json_url)
    .map_err(crate::http::Error::from)?
    .error_for_status()
    .map_err(crate::http::Error::from)?;
  let text = resp.text().map_err(crate::http::Error::from)?;
  let value: Vec<Release> =
    serde_json::from_str(&text[..]).map_err(|cause| DecodeError::from_serde(text, cause))?;
  Ok(value)
}

pub fn list(repo_urls: Vec<&str>) -> Result<Vec<Release>, Error> {
  let mut releases = Vec::new();
  for repo in repo_urls {
    let mut repo_releases = list_internal(repo)?;
    releases.append(&mut repo_releases);
  }
  Ok(releases)
}

fn latest_internal(repo_url: &str) -> Result<Release, crate::http::Error> {
  let base_url = repo_url.trim_end_matches('/');
  let index_json_url = format_url(base_url, "releases/latest");
  let resp = handle_github_rate_limit(crate::http::get(&index_json_url)?);
  let value: Release = resp.json()?;
  Ok(value)
}

pub fn latest(repo_urls: Vec<&str>) -> Result<Release, crate::http::Error> {
  let mut picked: Option<Release> = None;
  for repo in repo_urls {
    let release = latest_internal(repo)?;
    // Skip nightly releases when picking the latest
    if release.tag_name.is_nightly() {
      continue;
    }
    if let Some(ref picked_release) = picked {
      if release.tag_name > picked_release.tag_name {
        picked = Some(release);
      }
    } else {
      picked = Some(release);
    }
  }
  Ok(picked.expect("Can't get the latest release from the repositories"))
}

fn get_by_tag_internal(repo_url: &str, tag: &String) -> Result<Release, crate::http::Error> {
  let base_url = repo_url.trim_end_matches('/');
  let index_json_url = format_url(base_url, &format!("releases/tags/{tag}"));
  let resp = handle_github_rate_limit(crate::http::get(&index_json_url)?);
  let value: Release = resp.json()?;
  Ok(value)
}

pub fn get_by_tag(repo_urls: Vec<&str>, tag: &String) -> Result<Release, crate::http::Error> {
  let mut last_error = None;
  for repo in repo_urls {
    match get_by_tag_internal(repo, tag) {
      Ok(release) => return Ok(release),
      Err(e) => last_error = Some(e),
    }
  }
  Err(last_error.expect("Can't get the release by tag from the repositories"))
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  fn create_test_release(version: &str) -> Release {
    Release {
      tag_name: Version::parse(version).unwrap(),
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
          ("pact-4.13-linux-x64.tar.gz", true),
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
  fn test_asset_fallback_to_x64() {
    let mut release = create_test_release("4.13.0");
    release.assets = vec![Asset {
      url: Url::parse("https://example.com/asset").unwrap(),
      browser_download_url: Url::parse("https://example.com/download").unwrap(),
      id: 1,
      name: "pact-4.13.0-linux-20.04.tar.gz".to_string(), // No explicit arch
      size: 1000,
      created_at: DateTime::from_timestamp(0, 0).unwrap(),
      updated_at: DateTime::from_timestamp(0, 0).unwrap(),
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
      url: Url::parse("https://example.com/asset").unwrap(),
      browser_download_url: Url::parse("https://example.com/download").unwrap(),
      id: 1,
      name: asset_name.to_string(),
      size: 1000,
      created_at: DateTime::from_timestamp(0, 0).unwrap(),
      updated_at: DateTime::from_timestamp(0, 0).unwrap(),
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
      .find(|x| x.tag_name == expected_version)
      .map(|x| x.tag_name);
    assert_eq!(release, Some(expected_version));
    assert!(!release.unwrap().is_nightly());

    let repo = "kadena-io/pact-5";
    let expected_version = Version::parse("nightly").unwrap();
    let mut versions = list_internal(repo).expect("Can't get HTTP data");
    let release = versions
      .drain(..)
      .find(|x| x.tag_name == expected_version)
      .map(|x| x.tag_name);
    assert_eq!(release, Some(expected_version));
    assert!(release.unwrap().is_nightly());
  }
}
