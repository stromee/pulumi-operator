use std::path::PathBuf;

use git2::Cred;
use k8s_openapi::api::core::v1::Secret;
use kube::core::ObjectMeta;
use pulumi_operator_base::Inst;
use pulumi_operator_kubernetes::{
  kubernetes::service::KubernetesService,
  stack::source::git::inner::{InnerGitStackSourceSpec, GitAuthType},
};
use springtime_di::Component;
use thiserror::Error;

#[derive(Component)]
pub struct GitService {
  kubernetes_service: Inst<KubernetesService>,
}

#[derive(Debug, Error)]
pub enum GitError {
  #[error("Kubernetes error: {0}")]
  Kubernetes(#[from] kube::Error),

  #[error("Provided secret doesn't contain necessary data")]
  DataEmpty,

  #[error("Failed to parse UTF-8 string from secret data: {0}")]
  Utf8(#[from] std::string::FromUtf8Error),

  #[error("Git error: {0}")]
  Git(#[from] git2::Error),
}

impl GitService {
  pub async fn fetch(
    &self,
    spec: &InnerGitStackSourceSpec,
    metadata: &ObjectMeta,
  ) -> Result<impl Into<PathBuf>, GitError> {

    let cred = match spec.auth.as_ref() {
      Some(auth) => {
        let namespace = metadata.namespace.clone().map;
        let secret = self.kubernetes_service.get_in_namespace::<Secret>(namespace, &auth.secret_ref).await?;

        Some(match auth.kind {
          GitAuthType::Ssh => {
           

            todo!()
          }
          GitAuthType::Basic => {
            let data = secret.data.ok_or_else(|| GitError::DataEmpty)?;
            let username = data.get("username").ok_or_else(|| GitError::DataEmpty)?.clone();
            let username = String::from_utf8(username.0)?;
            let password = data.get("password").ok_or_else(|| GitError::DataEmpty)?.clone();
            let password = String::from_utf8(password.0)?;
            Cred::userpass_plaintext(&username, &password)?
          }
        })    
      },
      None => None,
    };
    Ok("/")
  }

}
