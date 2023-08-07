pub mod pulumi_execution;

use pulumi_cli::{PulumiCLI, UpOptions};
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
