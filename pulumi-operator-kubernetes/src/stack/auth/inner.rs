use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InnerStackAuthSpec {
  pub backend: String,
  pub backend_auth_secret: Option<String>,
  pub access_token_secret: Option<String>,
}