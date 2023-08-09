use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct InnerGitStackSourceSpec {
  pub repository: String,
  #[serde(rename = "ref")]
  pub git_ref: Option<String>,
  pub path: Option<String>,
}
