use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct InnerGitStackSourceSpec {
  pub repository: String,
  #[serde(rename = "ref")]
  pub git_ref: Option<String>,
  pub auth: Option<GitAuth>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GitAuth {
  pub kind: GitAuthType,
  pub secret_ref: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GitAuthType {
  Basic,
  Ssh,
}
