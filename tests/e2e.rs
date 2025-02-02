use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;
use tempfile::{tempdir, TempDir};

fn setup_test_env() -> TempDir {
  let temp_dir = tempdir().unwrap();

  // delete all files in the temp directory
  fs::remove_dir_all(temp_dir.path()).unwrap();

  // Create required directories
  fs::create_dir_all(temp_dir.path().join("pact-versions")).unwrap();
  fs::create_dir_all(temp_dir.path().join("pact-versions/.downloads")).unwrap();
  fs::create_dir_all(temp_dir.path().join("aliases")).unwrap();

  // Set required environment variables
  std::env::set_var("PACTUP_PACT_DIR", temp_dir.path());
  std::env::set_var("PACTUP_MULTISHELL_PATH", temp_dir.path().join("current"));

  temp_dir
}

// Update version numbers to use a known available version
const TEST_VERSION: &str = "4.13.0"; // Use a version we know exists
mod e2e_tests {}

#[test]
#[serial]
fn test_install_and_use_version() {
  let temp_dir = setup_test_env();

  // Install a specific version
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  cmd.arg("install").arg(TEST_VERSION).assert().success();

  // Verify installation directory exists
  assert!(temp_dir
    .path()
    .join(format!("pact-versions/v{}", TEST_VERSION))
    .exists());

  // Use the installed version
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  cmd
    .arg("use")
    .arg(TEST_VERSION)
    .env("PACTUP_MULTISHELL_PATH", temp_dir.path().join("current"))
    .assert()
    .success();
}

#[test]
#[serial]
fn test_alias_commands() {
  let temp_dir = setup_test_env();

  // Install version first
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  cmd.arg("install").arg(TEST_VERSION).assert().success();

  // Create alias
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  cmd
    .arg("alias")
    .arg(TEST_VERSION)
    .arg("stable")
    .assert()
    .success();

  // Verify alias exists
  assert!(temp_dir.path().join("aliases/stable").exists());

  // List aliases
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  let assert = cmd.arg("ls").assert();
  assert.success().stdout(predicate::str::contains("stable"));

  // Remove alias
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  cmd.arg("unalias").arg("stable").assert().success();

  // Verify alias was removed
  assert!(!temp_dir.path().join("aliases/stable").exists());
}

#[test]
#[serial]
fn test_default_version() {
  let temp_dir = setup_test_env();

  // Install version
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  cmd.arg("install").arg(TEST_VERSION).assert().success();

  // Set as default
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  cmd.arg("default").arg(TEST_VERSION).assert().success();

  // Verify default symlink exists
  assert!(temp_dir.path().join("aliases/default").exists());

  // Use default version
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  cmd
    .current_dir(&temp_dir)
    .env("PACTUP_MULTISHELL_PATH", temp_dir.path().join("current"))
    .arg("use")
    .arg("default")
    .assert()
    .success();

  // Check current version
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  cmd
    .arg("current")
    .env("PACTUP_MULTISHELL_PATH", temp_dir.path().join("current"))
    .assert()
    .success()
    .stdout(predicate::str::contains(TEST_VERSION));
}

#[test]
#[serial]
fn test_version_file() {
  let temp_dir = setup_test_env();

  // Create .pact-version file
  fs::write(temp_dir.path().join(".pact-version"), TEST_VERSION).unwrap();

  // Install version
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  cmd.arg("install").arg(TEST_VERSION).assert().success();

  // Use version from file
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  cmd
    .current_dir(&temp_dir)
    .env("PACTUP_MULTISHELL_PATH", temp_dir.path().join("current"))
    .arg("use")
    .assert()
    .success();

  // Verify correct version is used
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  cmd
    .current_dir(&temp_dir)
    .env("PACTUP_MULTISHELL_PATH", temp_dir.path().join("current"))
    .arg("current")
    .assert()
    .success()
    .stdout(predicate::str::contains(TEST_VERSION));
}

#[test]
#[serial]
fn test_which_command() {
  let _ = setup_test_env();

  // Install version
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  cmd.arg("install").arg(TEST_VERSION).assert().success();

  // Check which command output
  let mut cmd = Command::cargo_bin("pactup").unwrap();
  let assert = cmd.arg("which").arg(TEST_VERSION).assert();
  assert
    .success()
    .stdout(predicate::str::contains(format!("v{}", TEST_VERSION)));
}

// Add a helper test to ensure temp directories are created correctly
#[test]
#[serial]
fn test_setup_directories() {
  let temp_dir = setup_test_env();

  assert!(temp_dir.path().join("pact-versions").exists());
  assert!(temp_dir.path().join("pact-versions/.downloads").exists());
  assert!(temp_dir.path().join("aliases").exists());
}
