use crate::Inst;
use springtime_di::Component;

use crate::kubernetes::service::KubernetesService;

use super::{cluster_crd::ClusterStackAuth, crd::StackAuth};

#[derive(Component)]
pub struct StackAuthRepository {
  kubernetes_service: Inst<KubernetesService>,
}

impl StackAuthRepository {
  pub async fn get_namespaced_by_name_and_namespace(
    &self,
    name: impl ToString,
    namespace: impl ToString,
  ) -> Result<StackAuth, kube::Error> {
    self
      .kubernetes_service
      .get_in_namespace(namespace, name)
      .await
  }

  pub async fn get_by_name(
    &self,
    name: impl ToString,
  ) -> Result<ClusterStackAuth, kube::Error> {
    self.kubernetes_service.get(name).await
  }
}
