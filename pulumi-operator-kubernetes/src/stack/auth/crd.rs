use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::inner::InnerStackAuthSpec;

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
