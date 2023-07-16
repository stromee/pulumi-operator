use crate::stack::cached_pulumi_stack::CachedPulumiStack;
use async_trait::async_trait;
use springtime_di::injectable;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PulumiStackServiceError {
  #[error("pulumi task cancellation failed")]
  CancelFailed,
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
