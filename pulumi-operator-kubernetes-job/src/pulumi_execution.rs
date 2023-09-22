use futures::task::Spawn;
use k8s_openapi::api::core::v1::Secret;
use pulumi_cli::{CancelOptions, LoginOptions, PulumiCLI, UpOptions};
use pulumi_operator_kubernetes::kubernetes::service::KubernetesService;
use pulumi_operator_kubernetes::stack::auth::inner::InnerStackAuthSpec;
use pulumi_operator_kubernetes::stack::auth::repository::StackAuthRepository;
use pulumi_operator_kubernetes::stack::crd::{
  PulumiStack, StackAuthRefType, StackSourceRefType,
};
use pulumi_operator_kubernetes::stack::source::git::repository::GitStackSourceRepository;
use pulumi_operator_kubernetes::stack::source::oci::repository::OciStackSourceRepository;
use pulumi_operator_kubernetes::stack::source::Source;
use pulumi_operator_kubernetes::Inst;
use serde::Deserialize;
use springtime::runner::ApplicationRunner;
use springtime_di::future::{BoxFuture, FutureExt};
use springtime_di::instance_provider::ErrorPtr;
use springtime_di::{component_alias, Component};
use std::env::VarError;
use std::fs::read_to_string;
use std::sync::Arc;
use thiserror::Error;
use tokio::process::Command;

use crate::fetch_service::{FetchError, FetchService};

#[derive(Component)]
pub struct PulumiExecution {
  kubernetes_service: Inst<KubernetesService>,
  git_stack_source_repository: Inst<GitStackSourceRepository>,
  oci_stack_source_repository: Inst<OciStackSourceRepository>,
  stack_auth_repository: Inst<StackAuthRepository>,
  fetch_servcice: Inst<FetchService>,
}

#[derive(Debug, Error)]
pub enum PulumiExecutionError {
  #[error("Pulumi Stack Name is not defined")]
  PulumiStackNameNotDefined(VarError),
  #[error("Current Pod Namespace is not defined")]
  CurrentNamespaceNotDefined(VarError),
  #[error("Pulumi Stack definition could not be fetched")]
  PulumiStackNotFound(#[from] kube::Error),
  #[error("Failed to fetch stack source: {0}")]
  StackSourceFetchFailed(#[from] FetchError),
}

#[derive(Deserialize)]
pub struct PulumiConfig {
  pub runtime: String,
}

impl PulumiExecution {
  pub async fn run_internal(&self) -> Result<(), PulumiExecutionError> {
    let pulumi_stack = self.get_stack().await?;
    let inner_stack_source = self.get_inner_stack_source(&pulumi_stack).await?;
    let inner_stack_auth = self.get_inner_stack_auth(&pulumi_stack).await?;

    let namespace = std::env::var("WATCH_NAMESPACE")
      .map_err(PulumiExecutionError::CurrentNamespaceNotDefined)?;
    let access_token = match &inner_stack_auth.access_token_secret {
      None => None,
      Some(secret_name) => Some(
        String::from_utf8(
          self
            .kubernetes_service
            .get_in_namespace::<Secret>(namespace.clone(), secret_name)
            .await
            .unwrap()
            .data
            .unwrap()
            .get("token")
            .unwrap()
            .0
            .clone(),
        )
        .unwrap(),
      ),
    };

    if let Some(secret_name) = &inner_stack_auth.backend_auth_secret {
      let data = self
        .kubernetes_service
        .get_in_namespace::<Secret>(namespace, secret_name)
        .await
        .unwrap()
        .data
        .unwrap();

      let access_key_id =
        String::from_utf8(data.get("AWS_ACCESS_KEY_ID").unwrap().0.clone())
          .unwrap();
      let default_region =
        String::from_utf8(data.get("AWS_DEFAULT_REGION").unwrap().0.clone())
          .unwrap();
      let secret_access_key =
        String::from_utf8(data.get("AWS_SECRET_ACCESS_KEY").unwrap().0.clone())
          .unwrap();

      std::env::set_var("AWS_ACCESS_KEY_ID", access_key_id);
      std::env::set_var("AWS_DEFAULT_REGION", default_region);
      std::env::set_var("AWS_SECRET_ACCESS_KEY", secret_access_key);
    }

    if let Some(access_token) = access_token {
      std::env::set_var("PULUMI_CONFIG_PASSPHRASE", access_token);
    }

    let working_dir = self
      .fetch_servcice
      .fetch(&inner_stack_source, &pulumi_stack.metadata)
      .await?;

    let working_dir = match &pulumi_stack.spec.path {
      None => working_dir,
      Some(path) => working_dir.join(path),
    };

    let pulumi_config: PulumiConfig = serde_yaml::from_str(
      read_to_string(working_dir.join("Pulumi.yaml"))
        .unwrap()
        .as_str(),
    )
    .unwrap();

    let pulumi = PulumiCLI::new(working_dir.clone());

    match pulumi_config.runtime.as_str() {
      "nodejs" => {
        pulumi
          .spawn({
            let mut command = Command::new("npm");
            command.arg("install");
            command
          })
          .await;
      }
      _ => {
        unimplemented!()
      }
    }

    pulumi
      .login(LoginOptions {
        url: inner_stack_auth.backend,
      })
      .await;

    pulumi
      .cancel(CancelOptions {
        stack: pulumi_stack.spec.stack_name.clone(),
      })
      .await;

    let exit = pulumi
      .up(UpOptions {
        stack: pulumi_stack.spec.stack_name.clone(),
        ..Default::default()
      })
      .await;

    dbg!(exit);

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
  ) -> Result<Source, PulumiExecutionError> {
    let source_ref = &pulumi_stack.spec.source;
    let name = pulumi_stack.metadata.name.clone().unwrap();
    let namespace = pulumi_stack.metadata.namespace.clone().unwrap();
    Ok(match source_ref.type_ {
      StackSourceRefType::Git => self
        .git_stack_source_repository
        .get_namespaced_by_name_and_namespace(&name, &namespace)
        .await?
        .spec
        .inner
        .into(),
      StackSourceRefType::ClusterGit => self
        .git_stack_source_repository
        .get_by_name(&name)
        .await?
        .spec
        .inner
        .into(),
      StackSourceRefType::Oci => self
        .oci_stack_source_repository
        .get_namespaced_by_name_and_namespace(&name, &namespace)
        .await?
        .spec
        .inner
        .into(),
      StackSourceRefType::ClusterOci => self
        .oci_stack_source_repository
        .get_by_name(&name)
        .await?
        .spec
        .inner
        .into(),
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
