use common::models::{User,Database};
use crate::database;
use common::protocol::Response;
pub fn register(user: User) -> Response {
    let mut db = database::load_database();

    let is_this_username_exists = database::find_user(&db, &user.username);
    match is_this_username_exists {
        Some(_) => Response::Error{message:"Username already exists".to_string()},
        None => {
            database::add_user(&mut db, user);
            database::save_database(&db);
            Response::Success{message:"User registered successfully".to_string()}
        }
    }
}
pub fn login(user: User) -> Result<User,String> {
    let db = database::load_database();
    let is_this_username_exists = database::find_user(&db, &user.username);
    match is_this_username_exists {
        Some(user_found) => {
            if user_found.password == user.password {
                Ok(user.clone())
            } 
            else {
                Err("Incorrect password".to_string())
            }
        }
        None => Err("user not found".to_string())
    }
}
