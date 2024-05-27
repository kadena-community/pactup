use log::debug;
use std::path::Path;
use tempfile::TempDir;
use walkdir::WalkDir;

/// Check if a directory is a nix cache directory.
/// by checking if path contains `nix/store`.
fn is_nix_cache_path(dir: &Path) -> bool {
  dir.to_string_lossy().contains("nix/store")
}

#[cfg(unix)]
fn ensure_executable(path: &Path) -> std::io::Result<()> {
  // ensure that the pact binary is executable
  use std::os::unix::fs::PermissionsExt;
  if path.exists() {
    let metadata = std::fs::metadata(path)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions)?;
  }
  Ok(())
}

#[cfg(not(unix))]
fn ensure_executable(path: &Path) -> std::io::Result<()> {
  Ok(())
}

fn ensure_bin_dir(path: &Path) -> std::io::Result<()> {
  // recursively find a binary called 'pact' in the temp directory
  // if it exists, move it to temp_dir/bin
  let bin_dir = path.join("bin");
  if bin_dir.exists() {
    return Ok(());
  }

  let mut found_pact = None;

  // find the pact binary by recursively searching the temp directory
  for entry in WalkDir::new(path) {
    let entry = entry?;
    if entry.file_name() == "pact" {
      found_pact = Some(entry.path().to_path_buf());
      break;
    }
  }
  if let Some(pact_path) = found_pact {
    // flatten nix cache structure
    if is_nix_cache_path(&pact_path) {
      let nix_store_path = pact_path.parent().unwrap().parent().unwrap();
      // flatten nix cache directories by moving all the folders in the nix store to temp directory root
      for entry in WalkDir::new(nix_store_path) {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
          continue;
        }
        let new_path = entry_path
          .to_str()
          .unwrap()
          .replace(nix_store_path.to_str().unwrap(), path.to_str().unwrap());
        // FIXME: this is a hack since nix store path are not writable so we had to copy the files
        std::fs::create_dir_all(new_path.rsplitn(2, '/').last().unwrap())?;
        std::fs::copy(entry_path, new_path)?;
      }
    } else {
      // regular tarball structure
      std::fs::create_dir_all(&bin_dir)?;
      let new_path = bin_dir.join("pact");
      // move the binary to the bin directory
      std::fs::rename(&pact_path, bin_dir.join("pact"))?;
      // make it executable
      ensure_executable(&new_path)?;
    }
  }

  Ok(())
}

/// A "work-in-progress" directory, which will "teleport" into the path
/// given in `target` only on successful, guarding from invalid state in the file system.
///
/// Underneath, it uses `fs::rename`, so make sure to make the `temp_dir` inside the same
/// mount as `target`. This is why we have the `new_in` constructor.
pub struct DirectoryPortal<P: AsRef<Path>> {
  temp_dir: TempDir,
  target: P,
}

impl<P: AsRef<Path>> DirectoryPortal<P> {
  /// Create a new portal which will keep the temp files in
  /// a subdirectory of `parent_dir` until teleporting to `target`.
  #[must_use]
  pub fn new_in(parent_dir: impl AsRef<Path>, target: P) -> Self {
    let temp_dir = TempDir::new_in(parent_dir).expect("Can't generate a temp directory");
    debug!("Created a temp directory in {:?}", temp_dir.path());
    Self { temp_dir, target }
  }

  pub fn teleport(self) -> std::io::Result<P> {
    ensure_bin_dir(self.temp_dir.path())?;
    std::fs::rename(&self.temp_dir, &self.target)?;
    Ok(self.target)
  }
}

impl<P: AsRef<Path>> std::ops::Deref for DirectoryPortal<P> {
  type Target = Path;
  fn deref(&self) -> &Self::Target {
    self.as_ref()
  }
}

impl<P: AsRef<Path>> AsRef<Path> for DirectoryPortal<P> {
  fn as_ref(&self) -> &Path {
    self.temp_dir.as_ref()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use tempfile::tempdir;

  #[test_log::test]
  fn test_portal() {
    let tempdir = tempdir().expect("Can't generate a temp directory");
    let portal = DirectoryPortal::new_in(std::env::temp_dir(), tempdir.path().join("subdir"));
    let new_file_path = portal.to_path_buf().join("README.md");
    std::fs::write(new_file_path, "Hello world!").expect("Can't write file");
    let target = portal.teleport().expect("Can't close directory portal");

    let file_exists: Vec<_> = target
      .read_dir()
      .expect("Can't read dir")
      .map(|x| x.unwrap().file_name().into_string().unwrap())
      .collect();

    assert_eq!(file_exists, vec!["README.md"]);
  }
}
