use pulumi_operator_base::Inst;
use springtime_di::Component;

use crate::kubernetes::kubernetes_service::KubernetesService;
use crate::stack::pulumi_stack_cluster_source_crd::ClusterStackSource;
use crate::stack::pulumi_stack_crd::{StackSourceRef, StackSourceRefType};
use crate::stack::pulumi_stack_inner_source::InnerStackSourceSpec;
use crate::stack::pulumi_stack_source_crd::StackSource;

#[derive(Component)]
pub struct StackSourceRepository {
  kubernetes_service: Inst<KubernetesService>,
}

impl StackSourceRepository {
  pub async fn get_namespaced_by_name_and_namespace(
    &self,
    name: impl ToString,
    namespace: impl ToString,
  ) -> Result<StackSource, kube::Error> {
    self
      .kubernetes_service
      .get_in_namespace(namespace, name)
      .await
  }

  pub async fn get_by_name(
    &self,
    name: impl ToString,
  ) -> Result<ClusterStackSource, kube::Error> {
    self.kubernetes_service.get(name).await
  }
}
