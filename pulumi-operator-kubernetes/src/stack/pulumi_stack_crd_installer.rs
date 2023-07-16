use std::sync::Arc;

use kube::CustomResourceExt;
use springtime::future::FutureExt;
use springtime::runner::ApplicationRunner;
use springtime_di::future::BoxFuture;
use springtime_di::instance_provider::ErrorPtr;
use springtime_di::{component_alias, Component};

use pulumi_operator_base::Inst;

use crate::kubernetes::kubernetes_service::{
  KubernetesCrdInstallError, KubernetesService,
};
use crate::stack::pulumi_stack_crd::PulumiStack as PulumiStackCrd;

#[derive(Component)]
pub struct PulumiStackCrdInstaller {
  kubernetes_service: Inst<KubernetesService>,
}

impl PulumiStackCrdInstaller {
  async fn run_internal(&self) -> Result<(), KubernetesCrdInstallError> {
    self
      .kubernetes_service
      .install_crd(PulumiStackCrd::crd())
      .await?;

    Ok(())
  }
}

#[component_alias]
impl ApplicationRunner for PulumiStackCrdInstaller {
  fn run(&self) -> BoxFuture<'_, Result<(), ErrorPtr>> {
    async { self.run_internal().await.map_err(|err| Arc::new(err) as _) }
      .boxed()
  }
}
