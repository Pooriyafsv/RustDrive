mod client;
use client::start_client;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_client().await;
    Ok(())
}