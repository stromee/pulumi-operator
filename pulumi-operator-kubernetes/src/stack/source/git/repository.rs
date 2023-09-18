use crate::Inst;
use springtime_di::Component;

use crate::kubernetes::service::KubernetesService;

use super::{cluster_crd::ClusterGitStackSource, crd::GitStackSource};

#[derive(Component)]
pub struct GitStackSourceRepository {
  kubernetes_service: Inst<KubernetesService>,
}

impl GitStackSourceRepository {
  pub async fn get_namespaced_by_name_and_namespace(
    &self,
    name: impl ToString,
    namespace: impl ToString,
  ) -> Result<GitStackSource, kube::Error> {
    self
      .kubernetes_service
      .get_in_namespace(namespace, name)
      .await
  }

  pub async fn get_by_name(
    &self,
    name: impl ToString,
  ) -> Result<ClusterGitStackSource, kube::Error> {
    self.kubernetes_service.get(name).await
  }
}
