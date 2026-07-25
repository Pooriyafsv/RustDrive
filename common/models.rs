use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
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
pub struct Fileinfo{
    pub filename: String,
    pub filesize: u64,
    pub owner: String,
}
pub type SessionManager = Arc<Mutex<HashMap<String, UserSession>>>;
#[derive(Debug, Deserialize, Serialize,Clone)]
pub struct DataBase{
    pub users:Vec<User>,
}