use crate::kubernetes::kubernetes_service::KubernetesService;
use crate::stack::pulumi_stack_auth_crd::StackAuth;
use crate::stack::pulumi_stack_cluster_auth::ClusterStackAuth;
use pulumi_operator_base::Inst;
use springtime_di::Component;

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
