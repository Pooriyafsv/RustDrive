use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    // Client -> Server
    Login {
        username: String,
        password: String,
    },
    Register {
        username: String,
        password: String,
    },
    Upload {
        session_id: String,
        filename: String,
        size: usize,
    },
    Download {
        session_id: String,
        filename: String,
    },
    Delete {
        session_id: String,
        filename: String,
    },
    List {
        session_id: String,
    },
    Logout {
        session_id: String,
    },
    Rename {
        session_id: String,
        old_name: String,
        new_name: String,
    },
    Ping,
}
#[derive(Serialize, Deserialize, Debug)]

pub enum Response {
    Success { message: String },

    Error { message: String },

    Login { session_id: String },

    FileList { files: String},
    
    Pong,

    LoginSuccess { session_id: String },
}