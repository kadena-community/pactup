use crate::arch::Arch;
use crate::version::Version;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Uploader {
  pub name: Option<String>,
  pub email: Option<String>,
  pub login: String,
  pub id: usize,
  pub node_id: String,
  pub avatar_url: Url,
  pub gravatar_id: Option<String>,
  pub url: Url,
  pub html_url: Url,
  pub followers_url: Url,
  pub following_url: Url,
  pub gists_url: Url,
  pub starred_url: Url,
  pub subscriptions_url: Url,
  pub organizations_url: Url,
  pub repos_url: Url,
  pub events_url: Url,
  pub received_events_url: Url,
  pub r#type: String,
  pub site_admin: bool,
  pub starred_at: Option<String>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Author {
  pub login: String,
  pub id: usize,
  pub node_id: String,
  pub avatar_url: Url,
  pub gravatar_id: String,
  pub url: Url,
  pub html_url: Url,
  pub followers_url: Url,
  pub following_url: Url,
  pub gists_url: Url,
  pub starred_url: Url,
  pub subscriptions_url: Url,
  pub organizations_url: Url,
  pub repos_url: Url,
  pub events_url: Url,
  pub received_events_url: Url,
  pub r#type: String,
  pub site_admin: bool,
  pub patch_url: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Asset {
  pub url: Url,
  pub browser_download_url: Url,
  pub id: usize,
  pub node_id: String,
  pub name: String,
  pub label: Option<String>,
  pub state: String,
  pub content_type: String,
  pub size: i64,
  pub download_count: i64,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub uploader: Option<Uploader>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Release {
  pub url: Url,
  pub html_url: Url,
  pub assets_url: Url,
  pub upload_url: String,
  pub tarball_url: Option<Url>,
  pub zipball_url: Option<Url>,
  pub id: usize,
  pub node_id: String,
  pub tag_name: Version,
  pub target_commitish: String,
  pub name: Option<String>,
  pub body: Option<String>,
  pub draft: bool,
  pub prerelease: bool,
  pub created_at: Option<DateTime<Utc>>,
  pub published_at: Option<DateTime<Utc>>,
  pub author: Option<Author>,
  pub assets: Vec<Asset>,
}
impl Release {
  #[cfg(target_os = "linux")]
  pub fn filename_for_version(&self, _arch: &Arch) -> String {
    let version = &self.tag_name;
    if version.is_nightly() {
      "pact-binary-bundle.ubuntu-latest".to_string()
    } else {
      format!(
        "pact-{pact_ver}-linux-22.04",
        pact_ver = &version.digits_only().unwrap(),
        // platform = crate::system_info::platform_name(),
        // arch = arch,
      )
    }
  }

  #[cfg(target_os = "macos")]
  fn filename_for_version(&self, arch: &Arch) -> String {
    let version = &self.tag_name;
    if version.is_nightly() {
      match arch {
        Arch::X64 => "pact-binary-bundle.macos-latest".to_string(),
        Arch::Arm64 => "pact-binary-bundle.macos-m1".to_string(),
        _ => unimplemented!(),
      }
    } else {
      match arch {
        Arch::X64 => format!(
          "pact-{pact_ver}-osx",
          pact_ver = &version.digits_only().unwrap(),
        ),
        Arch::Arm64 => format!(
          "pact-{pact_ver}-aarch64-osx",
          pact_ver = &version.digits_only().unwrap(),
        ),
        _ => unimplemented!(),
      }
    }
  }

  #[cfg(windows)]
  fn filename_for_version(&self, arch: &Arch) -> String {
    // format!(
    //   "pact-{pact_ver}-win-{arch}.zip",
    //   pact = &version,
    //   arch = arch,
    // )
    unimplemented!()
  }

  pub fn download_url(&self, arch: &Arch) -> Url {
    let name = self.filename_for_version(arch);
    let asset = self
      .assets
      .iter()
      .find(|x| x.name.starts_with(name.as_str()))
      .expect("Can't find asset");
    asset.browser_download_url.clone()
  }
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
    println!("GitHub rate limit exceeded. Please wait until {reset_time} to try again.");
    resp
  } else {
    resp
  }
}
/// Prints
///
/// ```rust
/// use crate::remote_pact_index::list;
/// ```
pub fn list(repo_url: &String) -> Result<Vec<Release>, crate::http::Error> {
  let index_json_url = format!("https://api.github.com/repos/{repo_url}/releases");
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
  let index_json_url = format!("https://api.github.com/repos/{repo_url}/releases/latest");
  let resp = handle_github_rate_limit(crate::http::get(&index_json_url)?);
  let value: Release = resp.json()?;
  Ok(value)
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_list() {
    let repo = "kadena-io/pact".to_string();
    let expected_version = Version::parse("4.11.0").unwrap();
    let mut versions = list(&repo).expect("Can't get HTTP data");
    assert_eq!(
      versions
        .drain(..)
        .find(|x| x.tag_name == expected_version)
        .map(|x| x.tag_name),
      Some(expected_version)
    );
  }
}
