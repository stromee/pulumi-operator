use springtime::application;
use springtime_di::instance_provider::ComponentInstancePtr;

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
