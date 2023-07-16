use k8s_openapi::api::core::v1::Pod;
use kube::client::{Client, ClientBuilder};
use kube::config::InferConfigError;
use kube::{Api, Config};
use springtime::future::FutureExt;
use springtime_di::future::BoxFuture;
use springtime_di::instance_provider::ErrorPtr;
use springtime_di::Component;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KubernetesClientError {
  #[error("could not find kubernetes cluster configuration")]
  KubeConfigNotPresent(#[from] InferConfigError),
  #[error("could not create kubernetes client from configuration")]
  ClientCreationFailed(#[from] kube::error::Error),
}

#[derive(Component)]
#[component(constructor = "KubernetesClientProvider::new")]
pub struct KubernetesClientProvider {
  #[component(ignore)]
  client: Client,
}

impl KubernetesClientProvider {
  async fn new_internal() -> Result<Self, KubernetesClientError> {
    let config = Config::infer().await?;
    let client = Client::try_from(config)?;
    Ok(Self { client })
  }

  pub async fn get(&self) -> Client {
    self.client.clone()
  }

  fn new() -> BoxFuture<'static, Result<Self, ErrorPtr>> {
    async { Self::new_internal().await.map_err(|err| Arc::new(err) as _) }
      .boxed()
  }
}
