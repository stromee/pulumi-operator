use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct InnerOciStackSourceSpec {
  pub url: String,
  pub tag: Option<String>,
}
