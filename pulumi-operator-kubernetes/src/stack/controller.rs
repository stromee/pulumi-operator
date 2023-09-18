use std::sync::Arc;

use crate::stack::controller_strategy::{
  KubernetesPulumiStackControllerStrategy, PulumiStackControllerStrategyError,
};
use crate::Inst;
use springtime::runner::ApplicationRunner;
use springtime_di::future::{BoxFuture, FutureExt};
use springtime_di::instance_provider::{ComponentInstancePtr, ErrorPtr};
use springtime_di::{component_alias, Component};

#[derive(Component)]
pub struct PulumiStackController {
  controller_strategy: Inst<KubernetesPulumiStackControllerStrategy>,
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

#[cfg(feature = "boot")]
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
