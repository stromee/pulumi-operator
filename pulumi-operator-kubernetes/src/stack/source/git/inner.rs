use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct InnerGitStackSourceSpec {
  pub repository: String,
  #[serde(rename = "ref")]
  pub git_ref: Option<String>,
  pub path: Option<String>,
  pub auth: Option<GitAuth>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct GitAuth {
  pub kind: GitAuthType,
  pub secret_ref: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub enum GitAuthType {
  Basic,
  Ssh,
}
