use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::batch::v1::Job;
use kube::api::{DeleteParams, PostParams, WatchEvent};
use pulumi_operator_base::stack::cached_stack::CachedPulumiStack;
use pulumi_operator_base::stack::service::{
  PulumiStackService, PulumiStackServiceError,
};
use pulumi_operator_base::Inst;
use serde_json::json;
use springtime_di::{component_alias, Component};
use std::time::Duration;
use tokio::time::timeout;

use crate::config_provider::ConfigProvider;
use crate::kubernetes::service::KubernetesService;

#[derive(Component)]
pub struct KubernetesPulumiStackService {
  kubernetes_service: Inst<KubernetesService>,
  config_provider: Inst<ConfigProvider>,
}

#[component_alias]
#[async_trait]
impl PulumiStackService for KubernetesPulumiStackService {
  async fn update_stack(
    &self,
    stack: CachedPulumiStack,
  ) -> Result<(), PulumiStackServiceError> {
    self.cancel_stack(stack.clone()).await?;
    let mut parts = stack.name.splitn(2, '/');
    let namespace = parts.next().unwrap();
    let name = parts.next().unwrap();

    let operator_namespace = self
      .config_provider
      .operator_namespace()
      .map_err(|e| PulumiStackServiceError::Config(Box::new(e)))?; // TODO

    let job = serde_json::from_value(json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": name,
            "namespace": "pulumi-operator"
        },
        "spec": {
            "template": {
                "metadata": {
                    "name": "pulumi"
                },
                "spec": {
                    "containers": [{
                        "name": "pulumi",
                        "image": "ghcr.io/stromee/pulumi-operator/pulumi-operator-kubernetes-job:1.0.8",
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
                        }],
                        "imagePullPolicy": "Always"
                    }],
                    "serviceAccountName": "superuser",
                    "restartPolicy": "Never"
                }
            },
            "backoffLimit": 100,
            "successfulJobsHistoryLimit": 1,
            "failedJobsHistoryLimit": 1
        }
    }))
    .map_err(|err| PulumiStackServiceError::UpdateFailed(err.into()))?;

    let api = self
      .kubernetes_service
      .all_in_namespace_api::<Job>("pulumi-operator")
      .await;

    api
      .create(&PostParams::default(), &job)
      .await
      .map_err(|err| PulumiStackServiceError::UpdateFailed(err.into()))?;
    dbg!("Update stack123");

    Ok(())
  }

  async fn cancel_stack(
    &self,
    stack: CachedPulumiStack,
  ) -> Result<(), PulumiStackServiceError> {
    let mut parts = stack.name.splitn(2, '/');
    let namespace = parts.next().unwrap();
    let name = parts.next().unwrap();
    let api = self
      .kubernetes_service
      .all_in_namespace_api::<Job>("pulumi-operator")
      .await;

    if api.get(name).await.is_err() {
      return Ok(());
    };

    api
      .delete(name, &DeleteParams::foreground().grace_period(15))
      .await
      .expect("todo");

    // Poll until the pod is deleted or timeout
    let mut stream = api
      .watch(&Default::default(), "0")
      .await
      .expect("Failed to watch pod")
      .boxed();

    let timeout_duration = Duration::from_secs(20);
    timeout(timeout_duration, async {
      while let Some(status) =
        stream.try_next().await.expect("Error while watching")
      {
        match status {
          WatchEvent::Deleted(job)
            if job.metadata.name == Some(name.clone().to_string()) =>
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
