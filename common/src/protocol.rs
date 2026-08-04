use serde::{Deserialize, Serialize};

use crate::models::{FileMetadata,Access};

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
        size: u64,
    },
    Download {
        session_id: String,
        filename: String,
        owner: String,
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
    Search {
        session_id: String,
        query: String,
    },
    Ping,
    ChangeAccess {
        session_id: String,
        filename: String,
        access: Access,
    }
}
#[derive(Serialize, Deserialize, Debug)]

pub enum Response {
    Success { message: String },

    Error { message: String },

    Login { session_id: String },

    FileList { files: Vec<FileMetadata>},
    
    Pong,

    LoginSuccess { session_id: String },

    ReadyForUpload ,

    ReadyForDownload { size: u64 },

    SearchResults { files: Vec<FileMetadata> },
}