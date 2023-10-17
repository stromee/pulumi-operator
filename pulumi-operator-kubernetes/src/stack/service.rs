use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::batch::v1::{CronJob, Job};
use k8s_openapi::api::core::v1::{Container, ServiceAccount};
use k8s_openapi::api::rbac::v1::{Role, RoleBinding};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{DeleteParams, Object, PostParams, WatchEvent};
use serde_json::json;
use springtime_di::{component_alias, Component};
use std::error::Error;
use std::time::Duration;
use thiserror::Error;
use tokio::time::timeout;

use crate::config_provider::ConfigProvider;
use crate::kubernetes::service::KubernetesService;
use crate::stack::crd::PulumiStack;
use crate::Inst;

#[derive(Debug, Error)]
pub enum PulumiStackServiceError {
  #[error("pulumi task cancellation failed")]
  CancelFailed,

  #[error("Configuration error: {0}")]
  Config(Box<dyn Error + Sync + Send>),

  #[error("pulumi stack update failed: {0}")]
  UpdateFailed(Box<dyn Error + Sync + Send>),
}

#[derive(Component)]
pub struct KubernetesPulumiStackService {
  kubernetes_service: Inst<KubernetesService>,
  config_provider: Inst<ConfigProvider>,
}

impl KubernetesPulumiStackService {
  pub(crate) async fn update_stack(
    &self,
    stack: PulumiStack,
  ) -> Result<(), PulumiStackServiceError> {
    self.cancel_stack(stack.clone()).await?;
    self.create_service_account(stack.clone()).await?;
    self.create_role(stack.clone()).await?;
    self.create_role_binding(stack.clone()).await?;

    let name = stack.metadata.name.unwrap();
    let namespace = stack.metadata.namespace.unwrap();
    let init_containers = stack.spec.init_containers;
    let extra_volumes = stack.spec.extra_volumes;
    let container_override = stack.spec.main_container;
    let pod_override = stack.spec.main_pod;

    let operator_namespace = self
      .config_provider
      .operator_namespace()
      .map_err(|e| PulumiStackServiceError::Config(Box::new(e)))?; // TODO

    let mut main_container: Container = serde_json::from_value(json!({
        "name": "pulumi",
        "image": "ghcr.io/stromee/pulumi-operator/pulumi-operator-kubernetes-job:1.0.25",
        "env": [{
            "name": "PULUMI_STACK",
            "value": name
        }, {
            "name": "WATCH_NAMESPACE",
            "value": namespace
        }, {
            "name": ConfigProvider::OPERATOR_NS_VAR,
            "value": operator_namespace
        }, {
            "name": "RUST_BACKTRACE",
            "value": "full"
        }, {
            "name": "RUST_LOG",
            "value": "trace"
        }],
        "imagePullPolicy": "Always"
    })).unwrap();

    if let Some(main_container_override) = container_override {
      if let Some(mut extra_volume_mounts) =
        main_container_override.extra_volume_mounts
      {
        let mut volume_mounts =
          main_container.volume_mounts.clone().unwrap_or_default();
        volume_mounts.append(&mut extra_volume_mounts);
        main_container.volume_mounts.replace(volume_mounts);
      }

      if let Some(mut extra_env) = main_container_override.extra_env {
        let mut env = main_container.env.clone().unwrap_or_default();
        env.append(&mut extra_env);
        main_container.env.replace(env);
      }
    }

    let pod_annotations = pod_override
      .as_ref()
      .and_then(|pod| pod.extra_annotations.clone());

    let job = serde_json::from_value(json!({
        "apiVersion": "batch/v1",
        "kind": "CronJob",
        "metadata": {
            "name": format!("pulumi-{}", name),
            "namespace": namespace.clone()
        },
        "spec": {
            "schedule": "* * * * *",
            "concurrencyPolicy": "Forbid",
            "jobTemplate": {
                "spec": {
                    "template": {
                        "metadata": {
                            "name": "pulumi",
                            "annotations": pod_annotations
                        },
                        "spec": {
                            "initContainers": init_containers,
                            "containers": [main_container],
                            "volumes": extra_volumes,
                            "serviceAccountName": &name,
                            "restartPolicy": "Never"
                        }
                    },
                    "successfulJobsHistoryLimit": 1,
                    "failedJobsHistoryLimit": 1
                },
            },
            "backoffLimit": 100,
            "successfulJobsHistoryLimit": 1,
            "failedJobsHistoryLimit": 1
        }
    }))
    .map_err(|err| PulumiStackServiceError::UpdateFailed(err.into()))?;

    let api = self
      .kubernetes_service
      .all_in_namespace_api::<CronJob>(namespace.clone())
      .await;

    api
      .create(&PostParams::default(), &job)
      .await
      .map_err(|err| PulumiStackServiceError::UpdateFailed(err.into()))?;

    Ok(())
  }

  async fn create_service_account(
    &self,
    stack: PulumiStack,
  ) -> Result<(), PulumiStackServiceError> {
    let namespace = stack.metadata.namespace.unwrap();
    let name = stack.metadata.name.clone().unwrap();

    let service_account: ServiceAccount = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "ServiceAccount",
        "metadata": {
            "name": &name,
            "namespace": &namespace
        }
    }))
    .unwrap();

    let api = self
      .kubernetes_service
      .all_in_namespace_api(&namespace)
      .await;
    match api.get(&name).await {
      Ok(_) => {
        api
          .replace(&name, &PostParams::default(), &service_account)
          .await
          .unwrap();
      }
      Err(_) => {
        api
          .create(&PostParams::default(), &service_account)
          .await
          .unwrap();
      }
    }
    Ok(())
  }

  async fn create_role(
    &self,
    stack: PulumiStack,
  ) -> Result<(), PulumiStackServiceError> {
    let namespace = stack.metadata.namespace.unwrap();
    let name = stack.metadata.name.clone().unwrap();

    let role: Role = serde_json::from_value(json!({
        "apiVersion": "rbac.authorization.k8s.io/v1",
        "kind": "Role",
        "metadata": {
            "name": name,
            "namespace": namespace
        },
        "rules": [{
            "apiGroups": ["*"],
            "resources": ["*"],
            "verbs": ["*"]
        }]
    }))
    .unwrap();

    let api = self
      .kubernetes_service
      .all_in_namespace_api(&namespace)
      .await;

    match api.get(&name).await {
      Ok(_) => {
        api
          .replace(&name, &PostParams::default(), &role)
          .await
          .unwrap();
      }
      Err(_) => {
        api.create(&PostParams::default(), &role).await.unwrap();
      }
    }
    Ok(())
  }

  async fn create_role_binding(
    &self,
    stack: PulumiStack,
  ) -> Result<(), PulumiStackServiceError> {
    let namespace = stack.metadata.namespace.unwrap();
    let name = stack.metadata.name.clone().unwrap();

    let role_binding: RoleBinding = serde_json::from_value(json!({
        "apiVersion": "rbac.authorization.k8s.io/v1",
        "kind": "RoleBinding",
        "metadata": {
            "name": name,
            "namespace": namespace
        },
        "subjects": [{
            "kind": "ServiceAccount",
            "name": name,
            "namespace": namespace
        }],
        "roleRef": {
            "apiGroup": "rbac.authorization.k8s.io",
            "kind": "Role",
            "name": name
        }
    }))
    .unwrap();

    let api = self
      .kubernetes_service
      .all_in_namespace_api(&namespace)
      .await;

    match api.get(&name).await {
      Ok(_) => {
        api
          .replace(&name, &PostParams::default(), &role_binding)
          .await
          .unwrap();
      }
      Err(_) => {
        api
          .create(&PostParams::default(), &role_binding)
          .await
          .unwrap();
      }
    }

    Ok(())
  }
  pub(crate) async fn cancel_stack(
    &self,
    stack: PulumiStack,
  ) -> Result<(), PulumiStackServiceError> {
    let namespace = stack.metadata.namespace.unwrap();
    let name = stack.metadata.name.unwrap();
    let api = self
      .kubernetes_service
      .all_in_namespace_api::<CronJob>(namespace.clone())
      .await;

    if api.get(&format!("pulumi-{}", &name)).await.is_err() {
      return Ok(());
    };

    api
      .delete(
        &format!("pulumi-{}", &name),
        &DeleteParams::foreground().grace_period(15),
      )
      .await
      .expect("todo");

    // Poll until the pod is deleted or timeout
    let mut stream = api
      .watch(&Default::default(), "0")
      .await
      .expect("Failed to watch pod")
      .boxed();

    let timeout_duration = Duration::from_secs(1800);
    timeout(timeout_duration, async {
      while let Some(status) =
        stream.try_next().await.expect("Error while watching")
      {
        match status {
          WatchEvent::Deleted(job)
            if job.metadata.name
              == Some(format!("pulumi-{}", &name).clone().to_string()) =>
          {
            break;
          }
          _ => continue,
        }
      }
    })
    .await
    .expect("todo");

    Ok(())
  }
}
