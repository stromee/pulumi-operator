use std::path::{Path, PathBuf};

use kube::core::ObjectMeta;
use pulumi_operator_kubernetes::stack::source::Source;
use pulumi_operator_kubernetes::Inst;
use springtime_di::Component;
use thiserror::Error;

use crate::{
  git::service::{GitError, GitService},
  oci::service::{OciError, OciService},
};

#[derive(Component)]
pub struct FetchService {
  git_service: Inst<GitService>,
  oci_service: Inst<OciService>,
}

#[derive(Debug, Error)]
pub enum FetchError {
  #[error("Failed to setup git repository: {0}")]
  Git(#[from] GitError),
  #[error("Failed to setup from oci image: {0}")]
  Oci(#[from] OciError),
}

impl FetchService {
  pub async fn fetch(
    &self,
    source: &Source,
    metadata: &ObjectMeta,
  ) -> Result<PathBuf, FetchError> {
    let pulumi_dir = match source {
      Source::Git(git_source) => {
        self.git_service.fetch(git_source, metadata).await?.into()
      }
      Source::Oci(oci_source) => {
        self.oci_service.fetch(oci_source, metadata).await?.into()
      }
    };

    Ok(pulumi_dir)
  }
}
