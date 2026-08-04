//use tokio::io::{AsyncReadExt, AsyncWriteExt};
mod server;
mod auth;
mod database;
mod sessionmanager;
use server::start_server;
use std::sync::Arc;
use tokio::sync::Mutex;
use common::protocol_io;
mod metadata;
mod ai;
use ai::Agent_c;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut agent = Arc::new(Mutex::new(Agent_c::new()));
    let mut sessions = Arc::new(Mutex::new(sessionmanager::SessionManager::new()));
    let mut metadata = Arc::new(Mutex::new(metadata::MetadataManager::new().await));
    start_server(sessions,metadata,agent).await;
    Ok(())
}