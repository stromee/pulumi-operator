use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::inner::InnerGitStackSourceSpec;

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[kube(
  group = "pulumi.stromee.de",
  version = "v1",
  kind = "GitStackSource",
  plural = "gitstacksources"
)]
#[kube(namespaced)]
#[serde(rename_all = "camelCase")]
pub struct GitStackSourceSpec {
  #[serde(flatten)]
  pub inner: InnerGitStackSourceSpec,
}
