use std::path::PathBuf;

pub fn path() -> PathBuf {
  let path_as_string = if cfg!(windows) {
    "Z:/_PACTUP_/Nothing/Should/Be/Here"
  } else {
    "/dev/null"
  };

  PathBuf::from(path_as_string)
}

pub fn display_name() -> &'static str {
  "system"
}
