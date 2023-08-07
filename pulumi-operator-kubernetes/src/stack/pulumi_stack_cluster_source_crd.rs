use crate::stack::pulumi_stack_inner_source::InnerStackSourceSpec;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[kube(
  group = "pulumi.stromee.de",
  version = "v1",
  kind = "ClusterStackSource",
  plural = "clusterstacksources"
)]
#[serde(rename_all = "camelCase")]
pub struct ClusterStackSourceSpec {
  #[serde(flatten)]
  pub inner: InnerStackSourceSpec,
}
