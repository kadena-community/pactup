#[cfg(target_os = "windows")]
pub fn platform_os() -> &'static str {
  "windows"
}

#[cfg(target_os = "macos")]
pub fn platform_os() -> &'static str {
  "darwin"
}

#[cfg(target_os = "linux")]
pub fn platform_os() -> &'static str {
  "linux"
}

#[cfg(all(
  target_pointer_width = "32",
  any(target_arch = "arm", target_arch = "aarch64")
))]
pub fn platform_arch() -> &'static str {
  "armv7l"
}

#[cfg(all(
  target_pointer_width = "32",
  not(any(target_arch = "arm", target_arch = "aarch64"))
))]
pub fn platform_arch() -> &'static str {
  "x86"
}

#[cfg(all(
  target_pointer_width = "64",
  any(target_arch = "arm", target_arch = "aarch64")
))]
pub fn platform_arch() -> &'static str {
  "arm64"
}

#[cfg(all(
  target_pointer_width = "64",
  not(any(target_arch = "arm", target_arch = "aarch64"))
))]
pub fn platform_arch() -> &'static str {
  "x64"
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Arch {
  X86,
  X64,
  Arm64,
  Armv7l,
  Ppc64le,
  Ppc64,
  S390x,
}

impl Default for Arch {
  fn default() -> Arch {
    match platform_arch().parse() {
      Ok(arch) => arch,
      Err(e) => panic!("{}", e.details),
    }
  }
}

impl std::str::FromStr for Arch {
  type Err = ArchError;
  fn from_str(s: &str) -> Result<Arch, Self::Err> {
    match s {
      "x86" => Ok(Arch::X86),
      "x64" => Ok(Arch::X64),
      "arm64" => Ok(Arch::Arm64),
      "armv7l" => Ok(Arch::Armv7l),
      "ppc64le" => Ok(Arch::Ppc64le),
      "ppc64" => Ok(Arch::Ppc64),
      "s390x" => Ok(Arch::S390x),
      unknown => Err(ArchError::new(&format!("Unknown Arch: {unknown}"))),
    }
  }
}

impl std::fmt::Display for Arch {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let arch_str = match self {
      Arch::X86 => String::from("x86"),
      Arch::X64 => String::from("x64"),
      Arch::Arm64 => String::from("arm64"),
      Arch::Armv7l => String::from("armv7l"),
      Arch::Ppc64le => String::from("ppc64le"),
      Arch::Ppc64 => String::from("ppc64"),
      Arch::S390x => String::from("s390x"),
    };

    write!(f, "{arch_str}")
  }
}

#[derive(Debug)]
pub struct ArchError {
  details: String,
}

impl ArchError {
  fn new(msg: &str) -> ArchError {
    ArchError {
      details: msg.to_string(),
    }
  }
}

impl std::fmt::Display for ArchError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.details)
  }
}

impl std::error::Error for ArchError {
  fn description(&self) -> &str {
    &self.details
  }
}

pub struct Platform {
  pub os: &'static str,
  pub arch: Arch,
}

impl Default for Platform {
  fn default() -> Platform {
    Platform {
      os: platform_os(),
      arch: Arch::default(),
    }
  }
}

impl std::fmt::Display for Platform {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{name}-{arch}", name = self.os, arch = self.arch)
  }
}

pub fn get_platform() -> Platform {
  Platform::default()
}
