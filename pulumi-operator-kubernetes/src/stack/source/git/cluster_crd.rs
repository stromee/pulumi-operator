use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::inner::InnerGitStackSourceSpec;

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[kube(
  group = "pulumi.stromee.de",
  version = "v1",
  kind = "ClusterGitStackSource",
  plural = "clustergitstacksources"
)]
#[serde(rename_all = "camelCase")]
pub struct ClusterGitStackSourceSpec {
  #[serde(flatten)]
  pub inner: InnerGitStackSourceSpec,
}
