use crate::stack::pulumi_stack_service::PulumiStackServiceError;
use async_trait::async_trait;
use springtime_di::injectable;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PulumiStackControllerStrategyError {
  #[error("could not check for changed stacks")]
  Unknown(#[from] Box<dyn std::error::Error + Send + Sync>),
  #[error("pulumi stack service error occurred")]
  Service(#[from] PulumiStackServiceError),
  #[error("could not execute controller update")]
  UpdateWatchFailed,
}

#[injectable]
#[async_trait]
pub trait PulumiStackControllerStrategy {
  async fn initialize(&self) -> Result<(), PulumiStackControllerStrategyError>;

  async fn update(&self) -> Result<(), PulumiStackControllerStrategyError>;
}
