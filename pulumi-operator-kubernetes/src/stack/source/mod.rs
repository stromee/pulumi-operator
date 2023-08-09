pub mod git;
pub mod oci;

#[derive(Debug, Clone)]
pub enum Source {
  Git(git::inner::InnerGitStackSourceSpec),
  Oci(oci::inner::InnerOciStackSourceSpec),
}

impl From<git::inner::InnerGitStackSourceSpec> for Source {
  fn from(inner: git::inner::InnerGitStackSourceSpec) -> Self {
    Source::Git(inner)
  }
}

impl From<oci::inner::InnerOciStackSourceSpec> for Source {
  fn from(inner: oci::inner::InnerOciStackSourceSpec) -> Self {
    Source::Oci(inner)
  }
}
