use std::path::PathBuf;

use kube::core::ObjectMeta;
use pulumi_operator_kubernetes::stack::source::oci::inner::InnerOciStackSourceSpec;
use springtime_di::Component;
use thiserror::Error;

#[derive(Component)]
pub struct OciService {}

#[derive(Debug, Error)]
pub enum OciError {}

impl OciService {
  pub async fn fetch(
    &self,
    spec: &InnerOciStackSourceSpec,
    metadata: &ObjectMeta,
  ) -> Result<impl Into<PathBuf>, OciError> {
    Ok("/")
  }
}
