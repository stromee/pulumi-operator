use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use super::inner::InnerOciStackSourceSpec;

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[kube(
  group = "pulumi.stromee.de",
  version = "v1",
  kind = "ClusterOciStackSource",
  plural = "clusterocistacksources"
)]
#[serde(rename_all = "camelCase")]
pub struct ClusterOciStackSourceSpec {
  #[serde(flatten)]
  pub inner: InnerOciStackSourceSpec,
}
