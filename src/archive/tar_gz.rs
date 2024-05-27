use super::extract::{Error, Extract};
use std::{io::Read, path::Path};

pub struct TarGz<R: Read> {
  response: R,
}

impl<R: Read> TarGz<R> {
  #[allow(dead_code)]
  pub fn new(response: R) -> Self {
    Self { response }
  }
}

impl<R: Read> Extract for TarGz<R> {
  fn extract_into<P: AsRef<Path>>(self, path: P) -> Result<(), Error> {
    let gz_stream = flate2::read::GzDecoder::new(self.response);
    let mut tar_archive = tar::Archive::new(gz_stream);
    tar_archive.unpack(path)?;
    Ok(())
  }
}
