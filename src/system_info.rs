#[cfg(target_os = "windows")]
pub fn platform_os() -> PlatformOS {
  OS::Windows
}

#[cfg(target_os = "macos")]
pub fn platform_os() -> PlatformOS {
  PlatformOS::MacOS
}

#[cfg(target_os = "linux")]
pub fn platform_os() -> PlatformOS {
  PlatformOS::Linux
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PlatformOS {
  Windows,
  MacOS,
  Linux,
}

impl std::fmt::Display for PlatformOS {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PlatformOS::Windows => write!(f, "windows"),
      PlatformOS::MacOS => write!(f, "macos"),
      PlatformOS::Linux => write!(f, "linux"),
    }
  }
}

impl Default for PlatformOS {
  fn default() -> PlatformOS {
    platform_os()
  }
}

impl std::str::FromStr for PlatformOS {
  type Err = PlatformError;
  fn from_str(s: &str) -> Result<PlatformOS, Self::Err> {
    match s {
      "windows" => Ok(PlatformOS::Windows),
      "macos" => Ok(PlatformOS::MacOS),
      "linux" => Ok(PlatformOS::Linux),
      unknown => Err(PlatformError::new(&format!("Unknown OS: {unknown}"))),
    }
  }
}

#[cfg(all(
  target_pointer_width = "32",
  any(target_arch = "arm", target_arch = "aarch64")
))]
pub fn platform_arch() -> PlatformArch {
  PlatformArch::Armv7l
}

#[cfg(all(
  target_pointer_width = "32",
  not(any(target_arch = "arm", target_arch = "aarch64"))
))]
pub fn platform_arch() -> PlatformArch {
  PlatformArch::X86
}

#[cfg(all(
  target_pointer_width = "64",
  any(target_arch = "arm", target_arch = "aarch64")
))]
pub fn platform_arch() -> PlatformArch {
  PlatformArch::Arm64
}

#[cfg(all(
  target_pointer_width = "64",
  not(any(target_arch = "arm", target_arch = "aarch64"))
))]
pub fn platform_arch() -> PlatformArch {
  PlatformArch::X64
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PlatformArch {
  X86,
  X64,
  Arm64,
  Armv7l,
  Ppc64le,
  Ppc64,
  S390x,
}

impl Default for PlatformArch {
  fn default() -> PlatformArch {
    platform_arch()
  }
}

impl std::str::FromStr for PlatformArch {
  type Err = PlatformError;
  fn from_str(s: &str) -> Result<PlatformArch, Self::Err> {
    match s {
      "x86" => Ok(PlatformArch::X86),
      "x64" => Ok(PlatformArch::X64),
      "arm64" => Ok(PlatformArch::Arm64),
      "armv7l" => Ok(PlatformArch::Armv7l),
      "ppc64le" => Ok(PlatformArch::Ppc64le),
      "ppc64" => Ok(PlatformArch::Ppc64),
      "s390x" => Ok(PlatformArch::S390x),
      unknown => Err(PlatformError::new(&format!("Unknown Arch: {unknown}"))),
    }
  }
}

impl std::fmt::Display for PlatformArch {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let arch_str = match self {
      PlatformArch::X86 => String::from("x86"),
      PlatformArch::X64 => String::from("x64"),
      PlatformArch::Arm64 => String::from("arm64"),
      PlatformArch::Armv7l => String::from("armv7l"),
      PlatformArch::Ppc64le => String::from("ppc64le"),
      PlatformArch::Ppc64 => String::from("ppc64"),
      PlatformArch::S390x => String::from("s390x"),
    };

    write!(f, "{arch_str}")
  }
}

#[derive(Debug)]
pub struct PlatformError {
  details: String,
}

impl PlatformError {
  fn new(msg: &str) -> PlatformError {
    PlatformError {
      details: msg.to_string(),
    }
  }
}

impl std::fmt::Display for PlatformError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.details)
  }
}

impl std::error::Error for PlatformError {
  fn description(&self) -> &str {
    &self.details
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Platform(pub PlatformOS, pub PlatformArch);

impl std::fmt::Display for Platform {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{os}-{arch}", os = self.0, arch = self.1)
  }
}

pub fn get_platform() -> Platform {
  Platform::default()
}
