use pulumi_operator_base::Inst;
use pulumi_operator_kubernetes::kubernetes::kubernetes_service::KubernetesService;
use pulumi_operator_kubernetes::stack::pulumi_stack_auth_repository::StackAuthRepository;
use pulumi_operator_kubernetes::stack::pulumi_stack_crd::{
  PulumiStack, StackAuthRefType, StackSourceRefType,
};
use pulumi_operator_kubernetes::stack::pulumi_stack_inner_auth::InnerStackAuthSpec;
use pulumi_operator_kubernetes::stack::pulumi_stack_inner_source::InnerStackSourceSpec;
use pulumi_operator_kubernetes::stack::pulumi_stack_source_repository::StackSourceRepository;
use springtime::runner::ApplicationRunner;
use springtime_di::future::{BoxFuture, FutureExt};
use springtime_di::instance_provider::{ComponentInstancePtr, ErrorPtr};
use springtime_di::{component_alias, Component};
use std::env::VarError;
use std::sync::Arc;
use thiserror::Error;

#[derive(Component)]
pub struct PulumiExecution {
  kubernetes_service: Inst<KubernetesService>,
  stack_source_repository: Inst<StackSourceRepository>,
  stack_auth_repository: Inst<StackAuthRepository>,
}

#[derive(Debug, Error)]
pub enum PulumiExecutionError {
  #[error("Pulumi Stack Name is not defined")]
  PulumiStackNameNotDefined(VarError),
  #[error("Current Pod Namespace is not defined")]
  CurrentNamespaceNotDefined(VarError),
  #[error("Pulumi Stack definition could not be fetched")]
  PulumiStackNotFound(#[from] kube::Error),
}

impl PulumiExecution {
  pub async fn run_internal(&self) -> Result<(), PulumiExecutionError> {
    let pulumi_stack = self.get_stack().await?;
    let inner_stack_source = self.get_inner_stack_source(&pulumi_stack).await?;
    let inner_stack_auth = self.get_inner_stack_auth(&pulumi_stack).await?;
    dbg!("Found inner stack {}", inner_stack_source);

    Ok(())
  }

  pub async fn get_stack(&self) -> Result<PulumiStack, PulumiExecutionError> {
    let pulumi_stack_name = std::env::var("PULUMI_STACK")
      .map_err(PulumiExecutionError::PulumiStackNameNotDefined)?;

    let namespace = std::env::var("WATCH_NAMESPACE")
      .map_err(PulumiExecutionError::CurrentNamespaceNotDefined)?;

    Ok(
      self
        .kubernetes_service
        .get_in_namespace(&namespace, &pulumi_stack_name)
        .await?,
    )
  }

  pub async fn get_inner_stack_auth(
    &self,
    pulumi_stack: &PulumiStack,
  ) -> Result<InnerStackAuthSpec, PulumiExecutionError> {
    let auth_ref = &pulumi_stack.spec.auth;
    let name = pulumi_stack.metadata.name.clone().unwrap();
    let namespace = pulumi_stack.metadata.namespace.clone().unwrap();

    Ok(match auth_ref.type_ {
      StackAuthRefType::Namespace => {
        self
          .stack_auth_repository
          .get_namespaced_by_name_and_namespace(&name, &namespace)
          .await?
          .spec
          .inner
      }
      StackAuthRefType::Cluster => {
        self
          .stack_auth_repository
          .get_by_name(&name)
          .await?
          .spec
          .inner
      }
    })
  }

  pub async fn get_inner_stack_source(
    &self,
    pulumi_stack: &PulumiStack,
  ) -> Result<InnerStackSourceSpec, PulumiExecutionError> {
    let source_ref = &pulumi_stack.spec.source;
    let name = pulumi_stack.metadata.name.clone().unwrap();
    let namespace = pulumi_stack.metadata.namespace.clone().unwrap();
    Ok(match source_ref.type_ {
      StackSourceRefType::Namespace => {
        self
          .stack_source_repository
          .get_namespaced_by_name_and_namespace(&name, &namespace)
          .await?
          .spec
          .inner
      }
      StackSourceRefType::Cluster => {
        self
          .stack_source_repository
          .get_by_name(&name)
          .await?
          .spec
          .inner
      }
    })
  }
}

#[component_alias]
impl ApplicationRunner for PulumiExecution {
  fn run(&self) -> BoxFuture<'_, Result<(), ErrorPtr>> {
    async { self.run_internal().await.map_err(|err| Arc::new(err) as _) }
      .boxed()
  }
}
