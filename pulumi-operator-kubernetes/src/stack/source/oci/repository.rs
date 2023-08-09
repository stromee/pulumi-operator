use pulumi_operator_base::Inst;
use springtime_di::Component;

use crate::kubernetes::service::KubernetesService;

use super::{cluster_crd::ClusterOciStackSource, crd::OciStackSource};

#[derive(Component)]
pub struct OciStackSourceRepository {
  kubernetes_service: Inst<KubernetesService>,
}

impl OciStackSourceRepository {
  pub async fn get_namespaced_by_name_and_namespace(
    &self,
    name: impl ToString,
    namespace: impl ToString,
  ) -> Result<OciStackSource, kube::Error> {
    self
      .kubernetes_service
      .get_in_namespace(namespace, name)
      .await
  }

  pub async fn get_by_name(
    &self,
    name: impl ToString,
  ) -> Result<ClusterOciStackSource, kube::Error> {
    self.kubernetes_service.get(name).await
  }
}
