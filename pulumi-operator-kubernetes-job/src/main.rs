pub mod fetch_service;
pub mod git;
pub mod oci;
pub mod pulumi_execution;

use springtime::application;

#[tokio::main(flavor = "current_thread")]
async fn main() {
  #[cfg(feature = "kubernetes")]
  pulumi_operator_kubernetes::bind();
  application::create_default()
    .unwrap()
    .run()
    .await
    .expect("could not start controller");
}
