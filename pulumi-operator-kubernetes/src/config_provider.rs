use std::env::VarError;

use springtime_di::Component;
use thiserror::Error;

#[derive(Component)]
pub struct ConfigProvider {}

#[derive(Debug, Error)]
pub enum ConfigError {
  #[error("Failed to read environment variable: {0}")]
  Var(#[from] VarError),
}

impl ConfigProvider {
  pub const OPERATOR_NS_VAR: &'static str = "OPERATOR_NAMESPACE";

  pub fn operator_namespace(&self) -> Result<String, ConfigError> {
    Ok(std::env::var(Self::OPERATOR_NS_VAR)?)
  }
}
