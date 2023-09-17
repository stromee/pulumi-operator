use base64::{decode, encode, Engine};
use std::{
  collections::BTreeMap,
  path::{Path, PathBuf},
};

use git2::cert::{CertHostkey, SshHostKeyType};
use git2::{
  build::RepoBuilder, CertificateCheckStatus, Cred, FetchOptions,
  RemoteCallbacks,
};
use k8s_openapi::{api::core::v1::Secret, ByteString};
use kube::core::ObjectMeta;
use pulumi_operator_base::Inst;
use pulumi_operator_kubernetes::{
  config_provider::{ConfigError, ConfigProvider},
  kubernetes::service::KubernetesService,
  stack::source::git::inner::{GitAuth, GitAuthType, InnerGitStackSourceSpec},
};
use springtime_di::Component;
use thiserror::Error;
use tokio::{
  runtime::Builder,
  sync::{mpsc, oneshot},
  task::LocalSet,
};

#[derive(Component)]
pub struct GitService {
  kubernetes_service: Inst<KubernetesService>,
  config_provider: Inst<ConfigProvider>,
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

  #[error("Config error: {0}")]
  Config(#[from] ConfigError),

  #[error("Failed to join git fetch task: {0}")]
  Join(#[from] tokio::task::JoinError),

  #[error("Failed to build runtime for git fetch: {0}")]
  Runtime(#[from] tokio::io::Error),

  #[error("Failed to communicate between threads: {0}")]
  Recv(#[from] oneshot::error::RecvError),
}

impl GitService {
  pub async fn fetch(
    &self,
    spec: &InnerGitStackSourceSpec,
    metadata: &ObjectMeta,
  ) -> Result<impl Into<PathBuf>, GitError> {
    let namespace = match &metadata.namespace {
      Some(ns) => ns.clone(),
      None => self.config_provider.operator_namespace()?,
    };

    let git_controller = GitController {
      kubernetes_service: self.kubernetes_service.clone(),
    };
    let spec = spec.clone();

    let (tx, rx) = oneshot::channel();
    let rt = Builder::new_current_thread().enable_all().build()?;

    std::thread::spawn(move || {
      let local = LocalSet::new();

      local.spawn_local(async move {
        let res = async move {
          let mut callback = RemoteCallbacks::new();

          if let Some(auth) = &spec.auth {
            let data = git_controller.get_secret(&namespace, &auth).await?;
            git_controller.build_callback(auth, &data, &mut callback)?;
          }

          let mut fo = FetchOptions::new();
          fo.remote_callbacks(callback);

          let mut builder = RepoBuilder::new();
          builder.fetch_options(fo);

          builder.clone(spec.repository.as_str(), Path::new("./source"))?;

          Ok::<&str, GitError>("./source")
        }
        .await;
        tx.send(res)
          .expect("Failed to communicate between threads.");
      });

      rt.block_on(local);
    });
    rx.await?
  }
}

struct GitController {
  kubernetes_service: Inst<KubernetesService>,
}

impl GitController {
  async fn get_secret(
    &self,
    namespace: &str,
    auth: &GitAuth,
  ) -> Result<BTreeMap<String, ByteString>, GitError> {
    let secret = self
      .kubernetes_service
      .get_in_namespace::<Secret>(namespace, &auth.secret_ref)
      .await?;

    let data = secret.data.ok_or_else(|| GitError::DataEmpty)?;

    Ok(data)
  }

  fn build_callback(
    &self,
    auth: &GitAuth,
    data: &BTreeMap<String, ByteString>,
    callback: &mut RemoteCallbacks<'_>,
  ) -> Result<(), GitError> {
    let fallback_username = match data.get("username") {
      Some(username) => String::from_utf8(username.clone().0)?,
      None => "git".into(),
    };

    match auth.kind {
      GitAuthType::Ssh => {
        // let publickey = match data.get("identity.pub") {
        //   Some(publickey) => Some(String::from_utf8(publickey.clone().0)?),
        //   None => None,
        // };

        //TODO: Somehow github cloning fails with explicit public key?
        let publickey: Option<String> = None;

        let privatekey = String::from_utf8(
          data
            .get("identity")
            .ok_or_else(|| GitError::DataEmpty)?
            .clone()
            .0,
        )?;

        let passphrase = match data.get("identity.pass") {
          Some(passphrase) => Some(String::from_utf8(passphrase.clone().0)?),
          None => None,
        };

        //todo: handle known hosts
        let remote_hash = data
          .get("remote.pub.sha256")
          .map(|known_hosts| String::from_utf8(known_hosts.clone().0));

        if let Some(Ok(remote_hash)) = remote_hash {
          callback.certificate_check(move |cert, hostname| {
            if let Some(hostkey) = cert.as_hostkey() {
              dbg!(&remote_hash, encode(hostkey.hash_sha256().unwrap());
              if remote_hash == encode(hostkey.hash_sha256().unwrap()) {
                return Ok(CertificateCheckStatus::CertificateOk);
              }
            }
            Err(git2::Error::from_str(&format!(
              "Certificate check failed for host: {}",
              hostname
            )))
          });
        }

        callback.credentials(move |_url, username_from_url, _allowed_types| {
          Cred::ssh_key_from_memory(
            username_from_url.unwrap_or(&fallback_username),
            publickey.as_deref(),
            &privatekey,
            passphrase.as_deref(),
          )
        });
      }
      GitAuthType::Basic => {
        let username = data
          .get("username")
          .ok_or_else(|| GitError::DataEmpty)?
          .clone();
        let username = String::from_utf8(username.0)?;
        let password = data
          .get("password")
          .ok_or_else(|| GitError::DataEmpty)?
          .clone();
        let password = String::from_utf8(password.0)?;

        callback.credentials(
          move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext(&username, &password)
          },
        );
      }
    }
    Ok(())
  }
}
