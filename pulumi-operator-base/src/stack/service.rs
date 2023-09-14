use std::error::Error;

use crate::stack::cached_stack::CachedPulumiStack;
use async_trait::async_trait;
use springtime_di::injectable;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PulumiStackServiceError {
  #[error("pulumi task cancellation failed")]
  CancelFailed,

  #[error("Configuration error: {0}")]
  Config(Box<dyn Error + Sync + Send>),

  #[error("pulumi stack update failed: {0}")]
  UpdateFailed(Box<dyn Error + Sync + Send>),
}

#[injectable]
#[async_trait]
pub trait PulumiStackService {
  async fn update_stack(
    &self,
    stack: CachedPulumiStack,
  ) -> Result<(), PulumiStackServiceError>;

  async fn cancel_stack(
    &self,
    stack: CachedPulumiStack,
  ) -> Result<(), PulumiStackServiceError>;
}
