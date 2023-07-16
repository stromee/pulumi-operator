use crate::kubernetes::kubernetes_service::KubernetesService;
use async_trait::async_trait;
use pulumi_operator_base::stack::cached_pulumi_stack::CachedPulumiStack;
use pulumi_operator_base::stack::pulumi_stack_service::{
  PulumiStackService, PulumiStackServiceError,
};
use pulumi_operator_base::Inst;
use springtime_di::{component_alias, Component};

#[derive(Component)]
pub struct KubernetesPulumiStackService {
  kubernetes_service: Inst<KubernetesService>,
}

#[component_alias]
#[async_trait]
impl PulumiStackService for KubernetesPulumiStackService {
  async fn update_stack(
    &self,
    stack: CachedPulumiStack,
  ) -> Result<(), PulumiStackServiceError> {
    let mut parts = stack.name.splitn(2, '/');
    let namespace = parts.next().unwrap();
    let name = parts.next().unwrap();

    dbg!(namespace, name);
    Ok(())
  }

  async fn cancel_stack(
    &self,
    stack: CachedPulumiStack,
  ) -> Result<(), PulumiStackServiceError> {
    Ok(())
  }
}
