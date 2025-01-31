pub mod extract;
pub mod tar;
pub mod zip;

use std::io::Read;
use std::path::Path;

pub use self::extract::{Error, Extract};
use self::tar::Tar;
use self::zip::Zip;

pub enum Archive {
  Zip,
  TarXz,
  TarGz,
}
impl Archive {
  pub fn extract_archive_into(path: &Path, response: impl Read, url: &str) -> Result<(), Error> {
    let extractor: Box<dyn Extract> = match Archive::from_url(url) {
      Some(Self::Zip) => Box::new(Zip::new(response)),
      Some(Self::TarXz) => Box::new(Tar::Xz(response)),
      Some(Self::TarGz) => Box::new(Tar::Gz(response)),
      None => {
        return Err(Error::UnknownArchiveType {
          content_type: url.to_string(),
        })
      }
    };
    extractor.extract_into(path)?;
    Ok(())
  }

  pub fn from_url(url: &str) -> Option<Self> {
    if url.ends_with(".tar.xz") {
      Some(Self::TarXz)
    } else if url.ends_with(".tar.gz") {
      Some(Self::TarGz)
    } else if std::path::Path::new(url)
      .extension()
      .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
      Some(Self::Zip)
    } else {
      None
    }
  }

  pub fn supported() -> &'static [Self] {
    &[Self::TarXz, Self::TarGz, Self::Zip]
  }
}
