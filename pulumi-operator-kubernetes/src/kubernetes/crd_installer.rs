use std::sync::Arc;

use kube::CustomResourceExt;
use springtime::future::FutureExt;
use springtime::runner::ApplicationRunner;
use springtime_di::future::BoxFuture;
use springtime_di::instance_provider::ErrorPtr;
use springtime_di::{component_alias, Component};

use pulumi_operator_base::Inst;

use crate::kubernetes::service::{
  KubernetesCrdInstallError, KubernetesService,
};
use crate::stack::auth::cluster_crd::ClusterStackAuth as ClusterStackAuthCrd;
use crate::stack::auth::crd::StackAuth as StackAuthCrd;
use crate::stack::crd::PulumiStack as PulumiStackCrd;
use crate::stack::source::git::cluster_crd::ClusterGitStackSource as ClusterGitStackSourceCrd;
use crate::stack::source::git::crd::GitStackSource as GitStackSourceCrd;
use crate::stack::source::oci::cluster_crd::ClusterOciStackSource as ClusterOciStackSourceCrd;
use crate::stack::source::oci::crd::OciStackSource as OciStackSourceCrd;

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

    self
      .kubernetes_service
      .install_crd(ClusterGitStackSourceCrd::crd())
      .await?;

    self
      .kubernetes_service
      .install_crd(GitStackSourceCrd::crd())
      .await?;

    self
      .kubernetes_service
      .install_crd(ClusterOciStackSourceCrd::crd())
      .await?;

    self
      .kubernetes_service
      .install_crd(OciStackSourceCrd::crd())
      .await?;

    self
      .kubernetes_service
      .install_crd(ClusterStackAuthCrd::crd())
      .await?;

    self
      .kubernetes_service
      .install_crd(StackAuthCrd::crd())
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
