use crate::config::PactupConfig;
use crate::outln;
use colored::Colorize;

pub trait Command: Sized {
  type Error: std::error::Error;
  fn apply(self, config: &PactupConfig) -> Result<(), Self::Error>;

  fn handle_error(err: Self::Error, config: &PactupConfig) {
    let err_s = format!("{err}");
    outln!(config, Error, "{} {}", "error:".red().bold(), err_s.red());
    std::process::exit(1);
  }

  fn call(self, config: PactupConfig) {
    match self.apply(&config) {
      Ok(()) => (),
      Err(err) => Self::handle_error(err, &config),
    }
  }
}
