use crate::{
  system_info::{get_platform, Platform, PlatformArch, PlatformOS},
  version::Version,
};
use chrono::{DateTime, Utc};
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

impl Release {
  /// Returns the appropriate version matcher for the specified platform.
  pub fn version_matcher_for_platform(&self, platform: &Platform) -> Result<Regex, String> {
    let version = &self.tag_name;
    match platform {
      Platform(PlatformOS::Linux, PlatformArch::X64) => {
        let regex = if version.is_nightly() {
          // match the nightly version format for linux pact-binary-bundle.ubuntu-*.<tar.gz | zip>
          r"^(pact-binary-bundle\.ubuntu-latest|pact-nightly-linux(-\d+\.\d+)?)\.(tar\.gz|zip)$"
        } else {
          // match the stable version format for linux pact-<version>-<linux|ubuntu>-<ubuntu_version>.<tar.gz | zip>
          r"^pact-(\d+(\.\d+){0,2})(-(linux|ubuntu))?(-\d+\.\d+)?\.(tar\.gz|zip)$"
        };
        Regex::new(regex).map_err(|e| format!("Regex creation error: {e}"))
      }
      Platform(PlatformOS::MacOS, PlatformArch::X64) => {
        let regex = if version.is_nightly() {
          //  match the nightly version format for mac pact-binary-bundle.macos-latest.<tar.gz|zip>
          r"^(pact-binary-bundle\.macos-latest|pact-nightly-osx)\.(tar\.gz|zip)$"
        } else {
          // match the stable version format for mac pact-<version>-osx.<tar.gz | zip>
          r"^pact-(\d+(\.\d+){0,2})-osx\.(tar\.gz|zip)$"
        };
        Regex::new(regex).map_err(|e| format!("Regex creation error: {e}"))
      }
      Platform(PlatformOS::MacOS, PlatformArch::Arm64) => {
        let regex = if version.is_nightly() {
          //  match the nightly version format for mac pact-binary-bundle.macos-m1.<tar.gz|zip>
          r"^(pact-binary-bundle\.macos-m1|pact-nightly-aarch64-osx)\.(tar\.gz|zip)$"
        } else {
          // match the stable version format for mac pact-<version>-aarch64-osx.<tar.gz | zip>
          r"^pact-(\d+(\.\d+){0,2})-aarch64-osx\.(tar\.gz|zip)$"
        };
        Regex::new(regex).map_err(|e| format!("Regex creation error: {e}"))
      }
      _ => Err("Unsupported platform".to_string()),
    }
  }

  /// Finds the asset for the current architecture and platform.
  pub fn asset_for_current_platform(&self) -> Option<&Asset> {
    let platform = get_platform();
    let regex = self.version_matcher_for_platform(&platform).ok()?;
    self.assets.iter().find(|x| regex.is_match(&x.name))
  }

  /// Checks if the release has a supported asset for the current platform.
  pub fn has_supported_asset(&self) -> bool {
    self.asset_for_current_platform().is_some()
  }

  pub fn download_url(&self) -> Option<Url> {
    let asset = self.asset_for_current_platform();
    asset.map(|asset| asset.browser_download_url.clone())
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

fn format_url(repo_url: &String, path: &str) -> String {
  // i {
  //   // format!("https://ungh.cc/repos/{repo_url}/{path}",)
  // } else {
  format!("https://api.github.com/repos/{repo_url}/{path}")
  // }
}
/// Prints
///
/// ```rust
/// use crate::remote_pact_index::list;
/// ```
pub fn list(repo_url: &String) -> Result<Vec<Release>, crate::http::Error> {
  let index_json_url = format_url(repo_url, "releases");
  let resp = handle_github_rate_limit(crate::http::get(&index_json_url)?);
  let value: Vec<Release> = resp.json()?;
  Ok(value)
}

/// Prints
///
/// ```rust
/// use crate::remote_pact_index::latest;
/// ```
pub fn latest(repo_url: &String) -> Result<Release, crate::http::Error> {
  let index_json_url = format_url(repo_url, "releases/latest");
  let resp = handle_github_rate_limit(crate::http::get(&index_json_url)?);
  let value: Release = resp.json()?;
  Ok(value)
}

/// Prints
/// ```rust
/// use crate::remote_pact_index::get_by_tag;
///
pub fn get_by_tag(repo_url: &String, tag: &String) -> Result<Release, crate::http::Error> {
  let index_json_url = format_url(repo_url, &format!("releases/tags/{tag}"));
  let resp = handle_github_rate_limit(crate::http::get(&index_json_url)?);
  let value: Release = resp.json()?;
  Ok(value)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(target_os = "linux")]
  #[test]
  fn test_list() {
    let repo = "kadena-io/pact".to_string();
    let expected_version = Version::parse("4.11.0").unwrap();
    let mut versions = list(&repo).expect("Can't get HTTP data");
    let release = versions
      .drain(..)
      .find(|x| x.tag_name == expected_version)
      .map(|x| x.tag_name);
    assert_eq!(release, Some(expected_version));
    assert!(!release.unwrap().is_nightly());

    let repo = "kadena-io/pact-5".to_string();
    let expected_version = Version::parse("development-latest").unwrap();
    let mut versions = list(&repo).expect("Can't get HTTP data");
    let release = versions
      .drain(..)
      .find(|x| x.tag_name == expected_version)
      .map(|x| x.tag_name);
    assert_eq!(release, Some(expected_version));
    assert!(release.unwrap().is_nightly());
  }
  #[cfg(target_os = "linux")]
  #[test]
  fn test_get_by_tag() {
    let repo = "kadena-io/pact".to_string();
    let tag = "v4.11.0".to_string();
    let release = get_by_tag(&repo, &tag).expect("Can't get HTTP data");
    assert_eq!(release.tag_name.to_string(), tag);
    assert!(!release.tag_name.is_nightly());
    assert!(release.has_supported_asset());

    let repo = "kadena-io/pact-5".to_string();
    let tag = "development-latest".to_string();
    let release = get_by_tag(&repo, &tag).expect("Can't get HTTP data");
    assert_eq!(release.tag_name.to_string(), tag);
    assert!(release.tag_name.is_nightly());
    assert!(release.has_supported_asset());
  }

  #[cfg(target_os = "linux")]
  #[test]
  fn test_latest() {
    let repo = "kadena-io/pact".to_string();
    let release = latest(&repo).expect("Can't get HTTP data");
    assert!(!release.tag_name.is_nightly());
    assert!(release.has_supported_asset());

    let repo = "kadena-io/pact-5".to_string();
    let release = latest(&repo).expect("Can't get HTTP data");
    assert!(release.tag_name.is_nightly());
    assert!(release.has_supported_asset());
  }

  #[test]
  fn test_version_matcher_nightly_linux() {
    let release = Release {
      tag_name: Version::parse("development-latest").unwrap(),
      assets: vec![],
      prerelease: false,
      draft: false,
    };
    let platform = Platform(PlatformOS::Linux, PlatformArch::X64);
    let regex = release.version_matcher_for_platform(&platform).unwrap();

    // Old naming convention
    assert!(regex.is_match("pact-binary-bundle.ubuntu-latest.tar.gz"));
    // New naming convention
    assert!(regex.is_match("pact-nightly-linux.tar.gz"));
    assert!(regex.is_match("pact-nightly-linux-22.04.tar.gz"));
    // Should not match stable versions
    assert!(!regex.is_match("pact-4.11.0-linux.tar.gz"));
  }

  #[test]
  fn test_version_matcher_stable_linux() {
    let release = Release {
      tag_name: Version::parse("4.11.0").unwrap(),
      assets: vec![],
      prerelease: false,
      draft: false,
    };
    let platform = Platform(PlatformOS::Linux, PlatformArch::X64);
    let regex = release.version_matcher_for_platform(&platform).unwrap();

    assert!(regex.is_match("pact-4.11.0-linux.tar.gz"));
    assert!(regex.is_match("pact-4.11.0-linux-22.04.tar.gz"));
    assert!(regex.is_match("pact-4.11.0-ubuntu.tar.gz"));
    assert!(regex.is_match("pact-4.11.0-ubuntu-22.04.tar.gz"));
    // Should not match nightly builds
    assert!(!regex.is_match("pact-nightly-linux.tar.gz"));
    assert!(!regex.is_match("pact-binary-bundle.ubuntu-latest.tar.gz"));
  }

  #[test]
  fn test_version_matcher_nightly_macos_x64() {
    let release = Release {
      tag_name: Version::parse("development-latest").unwrap(),
      assets: vec![],
      prerelease: false,
      draft: false,
    };

    let platform = Platform(PlatformOS::MacOS, PlatformArch::X64);
    let regex = release.version_matcher_for_platform(&platform).unwrap();

    // Old naming convention
    assert!(regex.is_match("pact-binary-bundle.macos-latest.tar.gz"));
    // New naming convention
    assert!(regex.is_match("pact-nightly-osx.tar.gz"));
    // Should not match stable versions
    assert!(!regex.is_match("pact-4.11.0-osx.tar.gz"));
  }

  #[test]
  fn test_version_matcher_stable_macos_x64() {
    let release = Release {
      tag_name: Version::parse("4.11.0").unwrap(),
      assets: vec![],
      prerelease: false,
      draft: false,
    };

    let platform = Platform(PlatformOS::MacOS, PlatformArch::X64);
    let regex = release.version_matcher_for_platform(&platform).unwrap();

    assert!(regex.is_match("pact-4.11.0-osx.tar.gz"));
    // Should not match nightly builds
    assert!(!regex.is_match("pact-nightly-osx.tar.gz"));
    assert!(!regex.is_match("pact-binary-bundle.macos-latest.tar.gz"));
  }

  #[test]
  fn test_version_matcher_nightly_macos_arm64() {
    let release = Release {
      tag_name: Version::parse("development-latest").unwrap(),
      assets: vec![],
      prerelease: false,
      draft: false,
    };
    let platform = Platform(PlatformOS::MacOS, PlatformArch::Arm64);
    let regex = release.version_matcher_for_platform(&platform).unwrap();

    // Old naming convention
    assert!(regex.is_match("pact-binary-bundle.macos-m1.tar.gz"));
    // New naming convention
    assert!(regex.is_match("pact-nightly-aarch64-osx.tar.gz"));
    // Should not match stable versions
    assert!(!regex.is_match("pact-4.11.0-aarch64-osx.tar.gz"));
  }

  #[test]
  fn test_version_matcher_stable_macos_arm64() {
    let release = Release {
      tag_name: Version::parse("4.11.0").unwrap(),
      assets: vec![],
      prerelease: false,
      draft: false,
    };
    let platform = Platform(PlatformOS::MacOS, PlatformArch::Arm64);
    let regex = release.version_matcher_for_platform(&platform).unwrap();

    assert!(regex.is_match("pact-4.11.0-aarch64-osx.tar.gz"));
    // Should not match nightly builds
    assert!(!regex.is_match("pact-nightly-aarch64-osx.tar.gz"));
    assert!(!regex.is_match("pact-binary-bundle.macos-m1.tar.gz"));
  }
}
