use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize,Clone)]
pub struct User {
    pub username: String,
    pub password: String,
}
#[derive(Debug,Clone)]
pub struct UserSession {
    pub username: String,
    pub session_id: String,
}
#[derive(Debug, Deserialize, Serialize,Clone)]
pub enum Access {
    Public,
    Shared(Vec<String>),
    Private,
}
#[derive(Debug, Deserialize, Serialize,Clone)]
pub struct FileMetadata{
    pub filename: String,
    pub filesize: u64,
    pub owner: String,
    pub created_at: String,
    pub category: String,
    pub access: Access,
}
//pub type SessionManager = Arc<Mutex<HashMap<String, UserSession>>>;
#[derive(Debug, Deserialize, Serialize)]
pub struct Database{
    pub users:Vec<User>
}