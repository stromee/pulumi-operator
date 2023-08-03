use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::core::admission::{
  AdmissionRequest, AdmissionResponse, AdmissionReview,
};
use kube::runtime::controller::Action;
use kube::runtime::reflector::ObjectRef;
use kube::runtime::watcher::Config;
use kube::runtime::{watcher, Controller};
use kube::Resource;
use springtime_di::instance_provider::ComponentInstancePtr;
use springtime_di::{component_alias, Component};
use thiserror::Error;
use tokio::sync::Mutex;
use warp::Filter;

use pulumi_operator_base::stack::cached_pulumi_stack::CachedPulumiStack;
use pulumi_operator_base::stack::pulumi_stack_controller_strategy::{
  PulumiStackControllerStrategy, PulumiStackControllerStrategyError,
};
use pulumi_operator_base::stack::pulumi_stack_service::PulumiStackService;
use pulumi_operator_base::Inst;

use crate::kubernetes::kubernetes_service::KubernetesService;
use crate::stack::pulumi_stack_crd::PulumiStack;

const FINALIZER: &str = "pulumi.stromee.de";

type ControllerStream = Pin<
  Box<
    dyn Stream<
        Item = Result<
          (ObjectRef<PulumiStack>, Action),
          kube::runtime::controller::Error<
            PulumiStackControllerStrategyError,
            watcher::Error,
          >,
        >,
      > + Send,
  >,
>;

#[derive(Clone, Component)]
pub struct KubernetesPulumiStackControllerStrategy {
  kubernetes_service: Inst<KubernetesService>,
  stack_service: Inst<dyn PulumiStackService + Send + Sync>,
  #[component(default)]
  controller_stream: Arc<Mutex<Option<ControllerStream>>>,
}

impl KubernetesPulumiStackControllerStrategy {
  async fn handle_deletion(
    &self,
    stack: CachedPulumiStack,
  ) -> Result<(), PulumiStackControllerStrategyError> {
    self.stack_service.cancel_stack(stack).await?;
    Ok(())
  }

  async fn handle_creation(
    &self,
    stack: CachedPulumiStack,
  ) -> Result<(), PulumiStackControllerStrategyError> {
    self.stack_service.update_stack(stack).await?;
    Ok(())
  }

  async fn handle_update(
    &self,
    stack: CachedPulumiStack,
  ) -> Result<(), PulumiStackControllerStrategyError> {
    self.stack_service.update_stack(stack).await?;
    Ok(())
  }
}

impl KubernetesPulumiStackControllerStrategy {
  async fn reconcile(
    &self,
    stack: Arc<PulumiStack>,
  ) -> Result<Action, PulumiStackControllerStrategyError> {
    if !self
      .kubernetes_service
      .has_finalizer(stack.as_ref(), FINALIZER)
      .await
    {
      self
        .handle_creation(stack.as_ref().clone().try_into().map_err(Box::from)?)
        .await?;
      self
        .kubernetes_service
        .add_finalizer(stack.as_ref(), FINALIZER)
        .await
        .map_err(Box::from)?;
    } else if stack.meta().deletion_timestamp.is_some() {
      self
        .handle_deletion(stack.as_ref().clone().try_into().map_err(Box::from)?)
        .await?;
      self
        .kubernetes_service
        .remove_finalizer(stack.as_ref(), FINALIZER)
        .await
        .map_err(Box::from)?;
    } else {
      self
        .handle_update(stack.as_ref().clone().try_into().map_err(Box::from)?)
        .await?;
    }

    Ok(Action::await_change())
  }

  fn handle_error(
    &self,
    stack: Arc<PulumiStack>,
    error: &PulumiStackControllerStrategyError,
  ) -> Action {
    Action::requeue(Duration::from_secs(15))
  }
}

impl KubernetesPulumiStackControllerStrategy {
  async fn start_controller(
    &self,
  ) -> Result<(), PulumiStackControllerStrategyError> {
    let controller = Controller::new(
      self
        .kubernetes_service
        .all_in_handled_namespaces_api::<PulumiStack>()
        .await,
      Config::default().any_semantic(),
    )
    .shutdown_on_signal()
    .run(
      |stack, ctx| async move { ctx.reconcile(stack).await },
      |stack, error, ctx| ctx.handle_error(stack, error),
      Arc::new(self.clone()),
    );

    // self.start_admission_controller().await?;

    *self.controller_stream.lock().await = Some(Box::pin(controller) as _);
    Ok(())
  }

  async fn start_admission_controller(
    &self,
  ) -> Result<(), PulumiStackControllerStrategyError> {
    let routes = warp::path!("validate")
      .and(warp::body::json())
      .and_then(Self::validate)
      .with(warp::reply::with::header(
        "Content-Type",
        "application/json",
      ));

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
    Ok(())
  }

  async fn validate(
    ar: AdmissionReview<PulumiStack>,
  ) -> Result<impl warp::Reply, warp::Rejection> {
    let req = ar.request.unwrap();

    let Ok(admission_response) = Self::validate_request(req).await else {
      todo!()
    };

    Ok(warp::reply::json(&admission_response.into_review()))
  }

  async fn validate_request(
    req: AdmissionRequest<PulumiStack>,
  ) -> Result<AdmissionResponse, PulumiStackControllerStrategyError> {
    let mut admission_response = AdmissionResponse::from(&req);
    if let Some(pulumi_stack) = &req.object {
      if req.old_object.is_none() {
        dbg!("CREATING A NEW THINGY");
      }
    }
    Ok(admission_response)
  }
}

#[component_alias]
#[async_trait]
impl PulumiStackControllerStrategy for KubernetesPulumiStackControllerStrategy {
  async fn initialize(&self) -> Result<(), PulumiStackControllerStrategyError> {
    self.start_controller().await?;
    Ok(())
  }

  async fn update(&self) -> Result<(), PulumiStackControllerStrategyError> {
    let mut controller_stream = self.controller_stream.lock().await;
    let mut controller_stream = controller_stream
      .as_mut()
      .expect("controller still uninitialized");
    loop {
      controller_stream
        .next()
        .await
        .ok_or(PulumiStackControllerStrategyError::UpdateWatchFailed)?
        .map_err(Box::from)?;
    }
  }
}

#[derive(Debug, Error)]
pub enum PulumiStackConversionError {
  #[error("name is empty")]
  NameEmpty,
  #[error("namespace is empty")]
  NamespaceEmpty,
}

impl TryFrom<PulumiStack> for CachedPulumiStack {
  type Error = PulumiStackConversionError;

  fn try_from(k8s_stack: PulumiStack) -> Result<Self, Self::Error> {
    Ok(Self {
      name: format!(
        "{}/{}",
        k8s_stack
          .metadata
          .namespace
          .ok_or(PulumiStackConversionError::NamespaceEmpty)?,
        k8s_stack
          .metadata
          .name
          .ok_or(PulumiStackConversionError::NameEmpty)?
      ),
    })
  }
}
