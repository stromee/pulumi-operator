use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct InnerOciStackSourceSpec {
  pub url: String,
  pub tag: Option<String>,
}

