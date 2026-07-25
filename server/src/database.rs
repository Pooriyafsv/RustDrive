use common::models::{User, Database};
use std::fs;
use std::path::Path;
const PATH:&str = "data/Users.json";
pub fn load_database() -> Database {
    if !Path::new(PATH).exists(){
        return Database{users: Vec::new()};
    }
    let text = fs::read_to_string(PATH).unwrap();
    let database = serde_json::from_str::<Database>(&text).unwrap();
    database
}
pub fn save_database(database: &Database) {
    let json = serde_json::to_string_pretty(database).unwrap();
    fs::write(PATH,json).unwrap();
}
pub fn find_user(database: &Database, username: &str) -> Option<User> {
    for i in &database.users {
        if i.username == username {
            return Some(i.clone());
        }
    }
    return None;
}
pub fn add_user(database: &mut Database, user: User) {
    database.users.push(user);
}