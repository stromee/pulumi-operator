use std::sync::Arc;

use springtime::runner::ApplicationRunner;
use springtime_di::future::{BoxFuture, FutureExt};
use springtime_di::instance_provider::{ComponentInstancePtr, ErrorPtr};
use springtime_di::{component_alias, Component};

use crate::stack::controller_strategy::{
  PulumiStackControllerStrategy, PulumiStackControllerStrategyError,
};
use crate::Inst;

#[derive(Component)]
pub struct PulumiStackController {
  controller_strategy: Inst<dyn PulumiStackControllerStrategy + Send + Sync>,
}

impl PulumiStackController {
  async fn run_internal(
    &self,
  ) -> Result<(), PulumiStackControllerStrategyError> {
    self.controller_strategy.initialize().await?;
    loop {
      self.controller_strategy.update().await?;
    }
  }
}

#[component_alias]
impl ApplicationRunner for PulumiStackController {
  fn run(&self) -> BoxFuture<'_, Result<(), ErrorPtr>> {
    async { self.run_internal().await.map_err(|err| Arc::new(err) as _) }
      .boxed()
  }

  fn priority(&self) -> i8 {
    -1
  }
}
