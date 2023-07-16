use k8s_openapi::schemars::JsonSchema;
use kube::CustomResource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[kube(
  group = "pulumi.stromee.de",
  version = "v1",
  kind = "PulumiStack",
  plural = "pulumistacks"
)]
#[kube(namespaced)]
#[serde(rename_all = "camelCase")]
pub struct StackSpec {
  access_token_secret: String,
  backend: String,
  source: StackSourceRef,
  auth: StackAuthRef,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StackAuthRef {
  name: String,
  #[serde(rename = "type")]
  type_: StackAuthRefType,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum StackAuthRefType {
  #[serde(rename = "StackAuth")]
  Namespace,
  #[serde(rename = "ClusterStackAuth")]
  Cluster,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StackSourceRef {
  name: String,
  #[serde(rename = "type")]
  type_: StackSourceRefType,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum StackSourceRefType {
  #[serde(rename = "StackSource")]
  Namespace,
  #[serde(rename = "ClusterStackSource")]
  Cluster,
}
