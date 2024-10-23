use log::debug;

use super::extract::{Error, Extract};

use std::{io::Read, path::Path};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub enum Tar<R: Read> {
  /// Tar archive with XZ compression
  Xz(R),
  /// Tar archive with Gzip compression
  Gz(R),
}
impl<R: Read> Tar<R> {
  fn extract_into_impl<P: AsRef<Path>>(self, path: P) -> Result<(), Error> {
    debug!("Decompressing tar archive");
    let stream: Box<dyn Read> = match self {
      Self::Xz(response) => Box::new(xz2::read::XzDecoder::new(response)),
      Self::Gz(response) => Box::new(flate2::read::GzDecoder::new(response)),
    };
    let mut tar_archive = tar::Archive::new(stream);
    tar_archive.set_preserve_permissions(false);
    tar_archive.set_preserve_ownerships(false);
    tar_archive.set_overwrite(true);
    debug!("Extracting tar archive into {:?}", path.as_ref());
    // First, extract everything, even if the permissions are restrictive
    tar_archive.unpack(&path)?;

    debug!("Fixing permissions for extracted files and directories");
    // Now recursively set permissions for all directories and files
    fix_permissions_recursively(path.as_ref())?;
    Ok(())
  }
}
impl<R: Read> Extract for Tar<R> {
  fn extract_into(self: Box<Self>, path: &Path) -> Result<(), Error> {
    self.extract_into_impl(path)
  }
}

// Helper function to recursively fix permissions cross-platform
fn fix_permissions_recursively<P: AsRef<Path>>(path: P) -> Result<(), Error> {
  // Iterate over all files and directories recursively
  for entry in walkdir::WalkDir::new(path) {
    let entry = entry?;
    let path = entry.path();

    // Set permissions for Unix-like systems (Linux, macOS)
    #[cfg(unix)]
    {
      // Set permissions for directories
      if entry.file_type().is_dir() {
        let mut dir_permissions = std::fs::metadata(path)?.permissions();
        dir_permissions.set_mode(0o755); // Default directory permissions (rwxr-xr-x)
        std::fs::set_permissions(path, dir_permissions)?;
      }
      // Set permissions for regular files, preserving executable bits for all
      else if entry.file_type().is_file() {
        let metadata = std::fs::metadata(path)?;
        let mut file_permissions = metadata.permissions();

        // Default file permissions (rw-r--r--)
        let mut mode = 0o644;

        // Check if the file is executable (user, group, or others)
        if file_permissions.mode() & 0o111 != 0 {
          // If any of the executable bits are set, keep them for user, group, and others
          mode |= 0o111; // Retain all executable bits (for user, group, and others)
        }

        // Apply the computed mode
        file_permissions.set_mode(mode);
        std::fs::set_permissions(path, file_permissions)?;
      }
    }
  }

  Ok(())
}
