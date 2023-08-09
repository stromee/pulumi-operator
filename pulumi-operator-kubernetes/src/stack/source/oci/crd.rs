use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::inner::InnerOciStackSourceSpec;

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[kube(
  group = "pulumi.stromee.de",
  version = "v1",
  kind = "OciStackSource",
  plural = "ocistacksources",
  namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct OciStackSourceSpec {
  #[serde(flatten)]
  pub inner: InnerOciStackSourceSpec,
}
