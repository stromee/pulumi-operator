use springtime::application;
use springtime_di::instance_provider::ComponentInstancePtr;

pub mod config_provider;
pub mod kubernetes;
pub mod stack;

#[tokio::main(flavor = "current_thread")]
async fn main() {
  application::create_default()
    .unwrap()
    .run()
    .await
    .expect("could not start controller");
}

pub type Inst<T> = ComponentInstancePtr<T>;
