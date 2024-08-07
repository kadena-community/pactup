use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
struct EnginesField {
  pact: Option<node_semver::Range>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PackageJson {
  engines: Option<EnginesField>,
}

impl PackageJson {
  pub fn pact_range(&self) -> Option<&node_semver::Range> {
    self
      .engines
      .as_ref()
      .and_then(|engines| engines.pact.as_ref())
  }
}
