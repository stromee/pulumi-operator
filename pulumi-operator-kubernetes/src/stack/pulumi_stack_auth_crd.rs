use crate::stack::pulumi_stack_inner_auth::InnerStackAuthSpec;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[kube(
  group = "pulumi.stromee.de",
  version = "v1",
  kind = "StackAuth",
  plural = "stackauths"
)]
#[kube(namespaced)]
#[serde(rename_all = "camelCase")]
pub struct StackAuthSpec {
  #[serde(flatten)]
  pub inner: InnerStackAuthSpec,
}
