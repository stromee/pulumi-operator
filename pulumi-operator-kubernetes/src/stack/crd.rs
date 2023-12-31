use k8s_openapi::api::core::v1::{Container, EnvVar, Volume, VolumeMount};
use k8s_openapi::schemars::JsonSchema;
use kube::CustomResource;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::status::StackStatus;

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[kube(
  group = "pulumi.stromee.de",
  version = "v1",
  kind = "PulumiStack",
  plural = "pulumistacks",
  status = "StackStatus"
)]
#[kube(namespaced)]
#[serde(rename_all = "camelCase")]
pub struct StackSpec {
  pub stack_name: Option<String>,
  pub source: StackSourceRef,
  pub auth: StackAuthRef,
  pub path: Option<String>,
  pub init_containers: Option<Vec<Container>>,
  pub extra_volumes: Option<Vec<Volume>>,
  pub main_container: Option<MainContainerOverride>,
  pub main_pod: Option<MainPodOverride>,
  pub organization: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MainContainerOverride {
  pub extra_volume_mounts: Option<Vec<VolumeMount>>,
  pub extra_env: Option<Vec<EnvVar>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MainPodOverride {
  pub extra_annotations: Option<BTreeMap<String, String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StackAuthRef {
  pub name: String,
  #[serde(rename = "type")]
  pub type_: StackAuthRefType,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StackAuthRefType {
  #[serde(rename = "StackAuth")]
  Namespace,
  #[serde(rename = "ClusterStackAuth")]
  Cluster,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StackSourceRef {
  pub name: String,
  #[serde(rename = "type")]
  pub type_: StackSourceRefType,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StackSourceRefType {
  #[serde(rename = "GitStackSource")]
  Git,
  #[serde(rename = "ClusterGitStackSource")]
  ClusterGit,
  #[serde(rename = "OciStackSource")]
  Oci,
  #[serde(rename = "ClusterOciStackSource")]
  ClusterOci,
}
