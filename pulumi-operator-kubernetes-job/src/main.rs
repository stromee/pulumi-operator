use pulumi_cli::{PulumiCLI, UpOptions};

#[tokio::main]
async fn main() {
  let pulumi = PulumiCLI::new("test-stack");
  let output = pulumi
    .up(UpOptions {
      ..Default::default()
    })
    .await;
  println!("{}", String::from_utf8_lossy(&output.stdout));
}
