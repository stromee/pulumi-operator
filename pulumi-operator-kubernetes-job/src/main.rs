pub mod fetch_service;
pub mod git;
pub mod oci;
pub mod pulumi_execution;

use springtime::application;
use springtime_di::instance_provider::ComponentInstancePtr;

#[tokio::main(flavor = "current_thread")]
async fn main() {
  application::create_default()
    .unwrap()
    .run()
    .await
    .expect("could not start controller");
}
