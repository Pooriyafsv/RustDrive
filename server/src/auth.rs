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
pub fn login(user: User) -> Response {
    let db = database::load_database();
    let is_this_username_exists = database::find_user(&db, &user.username);
    match is_this_username_exists {
        Some(user) => {
            if user.password == user.password {
                Response::Success{message:"User logged in successfully".to_string()}
            } 
            else {
                Response::Error{message:"Incorrect password".to_string()}
            }
        }
        None => Response::Error{message:"Username does not exist".to_string()}
    }
}
