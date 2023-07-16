use crate::kubernetes::kubernetes_client_provider::KubernetesClientProvider;
use k8s_openapi::api::core::v1::{Namespace, Pod};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use k8s_openapi::NamespaceResourceScope;
use kube::api::{ObjectList, Patch, PatchParams, PostParams};
use kube::{Api, Resource, ResourceExt};
use pulumi_operator_base::Inst;
use serde::de::DeserializeOwned;
use springtime_di::Component;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Component)]
pub struct KubernetesService {
  client_provider: Inst<KubernetesClientProvider>,
}

#[derive(Debug, Error)]
pub enum KubernetesCrdInstallError {
  #[error("error occurred while communicating with kubernetes api")]
  KubernetesApiError(#[from] kube::Error),
  #[error("crd name is invalid or empty")]
  CrdNameInvalid,
}

impl KubernetesService {
  pub async fn install_crd(
    &self,
    crd: CustomResourceDefinition,
  ) -> Result<CustomResourceDefinition, KubernetesCrdInstallError> {
    let crd_api = Api::all(self.client_provider.get().await);
    // create crd
    match crd_api.create(&PostParams::default(), &crd).await {
      // creation succeeded
      Ok(resource) => {
        if let Some(name) = resource.metadata.name.clone() {
          // Poll until the CRD is available
          loop {
            match crd_api.get(&name).await {
              Ok(_) => break,
              Err(kube::Error::Api(error_response))
                if error_response.code == 404 =>
              {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
              }
              Err(err) => return Err(err.into()),
            }
          }
          tracing::info!("successfully created crd {}", name);
        }
        Ok(resource)
      }
      // already exists, patching if required
      Err(kube::Error::Api(error_response)) if error_response.code == 409 => {
        if let Some(name) = crd.metadata.name.clone() {
          tracing::debug!("crd {} already exists. trying to patch", name);
        }
        let resource_name = crd
          .metadata
          .name
          .clone()
          .ok_or(KubernetesCrdInstallError::CrdNameInvalid)?;

        let patch = Patch::Strategic(crd.clone());
        let patch_params = PatchParams::default();

        crd_api
          .patch(resource_name.as_str(), &patch_params, &patch)
          .await
          .map(|crd| {
            if let Some(name) = crd.metadata.name.clone() {
              // TODO: Implement polling for update
              tracing::info!("successfully updated crd {}", name);
            }
            crd
          })
          .map_err(|err| {
            if let Some(name) = crd.metadata.name.clone() {
              tracing::error!("failed to update crd {}", name);
            }
            err.into()
          })
      }
      // creation failed
      Err(error) => {
        if let Some(name) = crd.metadata.name.clone() {
          tracing::error!("failed to create crd {}", name);
        }

        Err(error.into())
      }
    }
  }

  pub async fn all_in_handled_namespaces_api<K>(&self) -> Api<K>
  where
    K: Resource<Scope = NamespaceResourceScope>
      + Clone
      + DeserializeOwned
      + Debug,
    <K as Resource>::DynamicType: Default,
  {
    // currently handle all namespaces
    // TODO: Make this configurable
    Api::all(self.client_provider.get().await)
  }

  pub async fn all<K>(&self) -> Result<ObjectList<K>, kube::Error>
  where
    K: Resource<Scope = NamespaceResourceScope>
      + Clone
      + DeserializeOwned
      + Debug,
    <K as Resource>::DynamicType: Default,
  {
    Api::all(self.client_provider.get().await)
      .list(&Default::default())
      .await
  }

  pub async fn all_in_handled_namespaces<K>(
    &self,
  ) -> Result<ObjectList<K>, kube::Error>
  where
    K: Resource<Scope = NamespaceResourceScope>
      + Clone
      + DeserializeOwned
      + Debug,
    <K as Resource>::DynamicType: Default,
  {
    // currently handle all namespaces
    // TODO: Make this configurable
    self.all().await
  }

  pub async fn all_in_namespace<K>(
    &self,
    namespace: &str,
  ) -> Result<ObjectList<K>, kube::Error>
  where
    K: Resource<Scope = NamespaceResourceScope>
      + Clone
      + DeserializeOwned
      + Debug,
    <K as Resource>::DynamicType: Default,
  {
    Api::namespaced(self.client_provider.get().await, namespace)
      .list(&Default::default())
      .await
  }

  pub async fn add_finalizer<K>(
    &self,
    resource: &K,
    finalizer: &str,
  ) -> Result<(), kube::Error>
  where
    K: Resource<DynamicType = (), Scope = NamespaceResourceScope>
      + Clone
      + DeserializeOwned
      + Debug,
    <K as Resource>::DynamicType: Default,
  {
    let client = self.client_provider.get().await;

    let api: Api<K> = if let Some(namespace) = &resource.namespace() {
      Api::namespaced(client.clone(), namespace)
    } else {
      Api::all(client.clone())
    };

    let patch = serde_json::json!({
        "metadata": {
            "finalizers": [finalizer]
        }
    });

    let patch_params = PatchParams::apply(finalizer);
    let _ = api
      .patch(
        &resource.meta().name.clone().expect("name is empty"),
        &patch_params,
        &Patch::Merge(&patch),
      )
      .await?;

    Ok(())
  }
  pub async fn remove_finalizer<K>(
    &self,
    resource: &K,
    finalizer: &str,
  ) -> Result<(), kube::Error>
  where
    K: Resource<DynamicType = (), Scope = NamespaceResourceScope>
      + Clone
      + DeserializeOwned
      + Debug,
    <K as Resource>::DynamicType: Default,
  {
    let client = self.client_provider.get().await;
    let api: Api<K> = if let Some(namespace) = &resource.namespace() {
      Api::namespaced(client.clone(), namespace)
    } else {
      Api::all(client.clone())
    };

    let patch = serde_json::json!({
        "metadata": {
            "finalizers": serde_json::Value::Null
        }
    });

    let patch_params = PatchParams::apply(finalizer);
    let _ = api
      .patch(
        &resource.meta().name.clone().expect("name is empty"),
        &patch_params,
        &Patch::Merge(&patch),
      )
      .await?;

    Ok(())
  }

  pub async fn has_finalizer<K>(&self, resource: &K, finalizer: &str) -> bool
  where
    K: Resource<DynamicType = ()> + Clone + DeserializeOwned + Debug,
    <K as Resource>::DynamicType: Default,
  {
    if let Some(finalizers) = resource.meta().finalizers.as_ref() {
      finalizers.iter().any(|f| f == finalizer)
    } else {
      false
    }
  }
}
