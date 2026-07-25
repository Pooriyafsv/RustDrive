//use tokio::io::{AsyncReadExt, AsyncWriteExt};
mod server;
mod auth;
mod database;
use server::start_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_server().await;
    Ok(())
}