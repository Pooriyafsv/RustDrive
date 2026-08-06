use common::models::Access::{self, Private};
//use log::Metadata;
use crate::ai::Agent_c;
use crate::auth;
use crate::metadata::MetadataManager;
use crate::sessionmanager::SessionManager;
use chrono::Utc;
use common::models::{FileMetadata, User};
use common::protocol::{Request, Response};
use common::protocol_io::{recv_json, send_json};
use std::sync::Arc;
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
pub async fn start_server(
    sessions: Arc<Mutex<SessionManager>>,
    metadata: Arc<Mutex<MetadataManager>>,
    agent: Arc<Mutex<Agent_c>>,
) {
    let listener = {
        match TcpListener::bind("127.0.0.1:8080").await {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("Failed to start server: {}", e);
                return;
            }
        }
    };
    println!("Server started on port 8080");

    loop {
        let (mut socket, address) = match listener.accept().await{
            Ok((socket,addres)) => (socket,addres),
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        };
        let sessions = sessions.clone();
        let metadata = metadata.clone();
        let agent = agent.clone();
        tokio::spawn(async move {
            println!("New connection: {}", address);
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            loop {
                let data: Request = match recv_json(&mut reader).await{
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Failed to read request: {}",e);
                        break;
                    }
                };
                match data {
                    Request::Ping => {
                        let res = Response::Pong;
                        //let res_j = serde_json::to_vec(&res).unwrap();
                        //println!("Sending response: {:?}",res_j);
                        //send_json(&mut writer, &res).await.unwrap();
                        if let Err(e) =send_json(&mut writer, &res).await{
                            eprintln!("Error sending response: {}",e);
                            break;
                        }
                    }
                    Request::Register { username, password } => {
                        let user = User {
                            username: username,
                            password: password,
                        };
                        let response = auth::register(user);
                        //send_json(&mut writer, &response).await.unwrap();
                        if let Err(e) =send_json(&mut writer, &response).await{
                            eprintln!("Error sending response: {}",e);
                            break;
                        }
                    }
                    Request::Login { username, password } => {
                        let user = User {
                            username: username,
                            password: password,
                        };
                        let response = match auth::login(user) {
                            Ok(user) => {
                                let mut manager = sessions.lock().await;
                                let session_id = manager.create_session(user.username);
                                Response::LoginSuccess { session_id }
                            }
                            Err(e) => Response::Error { message: e },
                        };
                        //let res_j = serde_json::to_vec(&response).unwrap();
                        //writer.write_all(&res_j).await.unwrap();
                        //writer.flush().await.unwrap();
                        //send_json(&mut writer, &response).await.unwrap();
                        if let Err(e) =send_json(&mut writer, &response).await{
                            eprintln!("Error sending response: {}",e);
                            break;
                        }
                    }
                    Request::Upload {
                        session_id,
                        filename,
                        size,
                    } => {
                        {
                            let manager = sessions.lock().await;
                            if !manager.validate(session_id.clone()) {
                                let response = Response::Error {
                                    message: "Invalid session".to_string(),
                                };
                                //send_json(&mut writer, &response).await.unwrap();
                                if let Err(e) = send_json(&mut writer, &response).await {
                                    eprintln!("Error sending response: {}", e);
                                    break;
                                }
                                continue;
                            }
                        };
                        let username = {
                            let manager = sessions.lock().await;
                            manager.get_username(session_id.clone()).unwrap()
                        };
                        let dir = format!("storage/{}", username.clone());
                        fs::create_dir_all(&dir).await.unwrap();
                        let path = format!("{}/{}", dir, filename.clone());
                        let mut file = File::create(&path).await.unwrap();
                        let res = Response::ReadyForUpload;
                        //let res_j = serde_json::to_vec(&res).unwrap();
                        //writer.write_all(&res_j).await.unwrap();
                        //writer.flush().await.unwrap();
                        //send_json(&mut writer, &res).await.unwrap();
                        if let Err(e) = send_json(&mut writer, &res).await {
                            eprintln!("Error sending response: {}", e);
                            break;
                        }
                        let mut recived = 0;
                        let mut buf = [0; 4096];
                        while recived < size {
                            let read = reader.read(&mut buf).await.unwrap();
                            if read == 0 {
                                break;
                            }
                            file.write_all(&buf[..read]).await.unwrap();
                            recived += read as u64;
                        }
                        let res = Response::Success {
                            message: "upload complete".to_string(),
                        };
                        //let res_j = serde_json::to_vec(&res).unwrap();
                        //writer.write_all(&res_j).await.unwrap();
                        //writer.flush().await.unwrap();
                        //send_json(&mut writer, &res).await.unwrap();
                        if let Err(e) = send_json(&mut writer, &res).await {
                            eprintln!("Error sending response: {}", e);
                            break;
                        }
                        let file_info = FileMetadata {
                            filename: filename.clone(),
                            filesize: size,
                            owner: username,
                            created_at: Utc::now().format("%Y-%m-%d").to_string(),
                            category: {
                                let  manager = agent.lock().await;
                                match manager.classify(&filename).await {
                                    Ok(c) => c,
                                    Err(e) => {
                                        println!("Error: {}", e);
                                        "Unknown".to_string()
                                    }
                                }
                            },
                            access: Private,
                        };
                        {
                            let mut data_manager = metadata.lock().await;
                            data_manager.add(file_info).await;
                        }

                        // Handle the upload
                        // ...
                    }
                    Request::Download {
                        session_id,
                        filename,
                        owner,
                    } => {
                        {
                            let manager = sessions.lock().await;
                            if !manager.validate(session_id.clone()) {
                                let response = Response::Error {
                                    message: "Invalid session".to_string(),
                                };
                                //let res_j = serde_json::to_vec(&response).unwrap();
                                //writer.write_all(&res_j).await.unwrap();
                                //writer.flush().await.unwrap();

                                //send_json(&mut writer, &response).await.unwrap();
                                if let Err(e) = send_json(&mut writer, &response).await {
                                    eprintln!("Error sending response: {}", e);
                                    break;
                                }
                                continue;
                            }
                        }
                        let username = {
                            let manager = sessions.lock().await;
                            manager.get_username(session_id.clone()).unwrap()
                        };
                        let is_owner = owner == username;
                        let is_exists={
                            let manager = metadata.lock().await;
                            manager.find(&owner,&filename).is_some()
                        };
                        if !is_exists {
                            let res = Response::Error { message: "file not found".to_string() };
                                    if let Err(e) = send_json(&mut writer, &res).await {
                                        eprintln!("Error sending response: {}", e);
                                        break;

                                    }
                                    continue;
                        }
                        let accesse = {
                            let manager = metadata.lock().await;
                            manager.downloadable(&filename, &owner, is_owner)
                        };
                        if !accesse {
                            let res = Response::Error {
                                message: "access denied".to_string(),
                            };
                            match send_json(&mut writer, &res).await {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("Error sending response: {}", e);
                                    break;
                                }
                            }
                            continue;
                        }

                        let path = format!("storage/{}/{}", owner, filename);
                        if fs::metadata(&path).await.is_err() {
                            let res = Response::Error {
                                message: "File not found".to_string(),
                            };
                            //let res_j = serde_json::to_vec(&res).unwrap();
                            //writer.write_all(&res_j).await.unwrap();
                            //writer.flush().await.unwrap();
                            //send_json(&mut writer, &res).await.unwrap();
                            if let Err(e) = send_json(&mut writer, &res).await {
                                eprintln!("Error sending response: {}", e);
                                break;
                            }
                            continue;
                        }
                        let metadata = fs::metadata(&path).await.unwrap();
                        let size = metadata.len();
                        let res = Response::ReadyForDownload { size };
                        //let res_j= serde_json::to_vec(&res).unwrap();
                        //writer.write_all(&res_j).await.unwrap();
                        //writer.flush().await.unwrap();
                        //send_json(&mut writer, &res).await.unwrap();
                        if let Err(e) = send_json(&mut writer, &res).await {
                            eprintln!("Error sending response: {}", e);
                            break;
                        }
                        let mut file = File::open(path).await.unwrap();
                        let mut buffer = [0u8; 4096];
                        loop {
                            let bytes_read = file.read(&mut buffer).await.unwrap();
                            if bytes_read == 0 {
                                break;
                            }
                            writer.write_all(&buffer[..bytes_read]).await.unwrap();
                            writer.flush().await.unwrap();
                        }
                    }
                    Request::List { session_id } => {
                        {
                            let manager = sessions.lock().await;
                            if !manager.validate(session_id.clone()) {
                                let response = Response::Error {
                                    message: "Invalid session".to_string(),
                                };
                                //let res_j = serde_json::to_vec(&response).unwrap();
                                //writer.write_all(&res_j).await.unwrap();
                                //writer.flush().await.unwrap();
                                //send_json(&mut writer, &response).await.unwrap();
                                if let Err(e) = send_json(&mut writer, &response).await {
                                    eprintln!("Error sending response: {}", e);
                                    break;
                                }
                                continue;
                            }
                        }
                        let username = {
                            let manager = sessions.lock().await;
                            manager.get_username(session_id.clone()).unwrap()
                        };
                        //let path = format!("storage/{}",username);
                        //if fs::metadata(&path).await.is_err() {
                        //    let response = Response::Error {
                        //    message: "you don't have a storage folder".to_string(),
                        //    };
                        //let res_j = serde_json::to_vec(&response).unwrap();
                        //writer.write_all(&res_j).await.unwrap();
                        //writer.flush().await.unwrap();
                        //    send_json(&mut writer, &response).await.unwrap();

                        //    continue;
                        //}
                        //let mut dir = tokio::fs::read_dir(path).await.unwrap();
                        //let mut files = Vec::new();
                        //loop {
                        //    let entry = dir.next_entry().await.unwrap();
                        //    match entry {
                        //        Some(file) =>{
                        //            let name = file.file_name().to_string_lossy().to_string();
                        //            files.push(name);
                        //        }
                        //        None => break,
                        //    }
                        //}
                        let files = {
                            let manager = metadata.lock().await;
                            let mut file = manager.get_user_files(&username);
                            file.extend_from_slice(&manager.public_files());
                            file
                        };
                        let res = Response::FileList { files };
                        //let res_j = serde_json::to_vec(&res).unwrap();
                        //writer.write_all(&res_j).await.unwrap();
                        //writer.flush().await.unwrap();
                        //send_json(&mut writer, &res).await.unwrap();
                        if let Err(e) = send_json(&mut writer, &res).await {
                            eprintln!("Error sending response: {}", e);
                            break;
                        }
                    }
                    Request::Delete {
                        session_id,
                        filename,
                    } => {
                        {
                            let manager = sessions.lock().await;
                            if !manager.validate(session_id.clone()) {
                                let response = Response::Error {
                                    message: "Invalid session".to_string(),
                                };
                                //let res_j = serde_json::to_vec(&response).unwrap();
                                //writer.write_all(&res_j).await.unwrap();
                                //writer.flush().await.unwrap();
                                //send_json(&mut writer, &response).await.unwrap();
                                if let Err(e) = send_json(&mut writer, &response).await {
                                    eprintln!("Error sending response: {}", e);
                                    break;
                                }
                                continue;
                            }
                        }
                        let username = {
                            let manager = sessions.lock().await;
                            manager.get_username(session_id.clone()).unwrap()
                        };
                        let path = format!("storage/{}/{}", username, filename);
                        match tokio::fs::remove_file(path).await {
                            Ok(_) => {
                                let res = Response::Success {
                                    message: "File deleted succefuly".to_string(),
                                };
                                //let res_j = serde_json::to_vec(&res).unwrap();
                                //writer.write_all(&res_j).await.unwrap();
                                //writer.flush().await.unwrap();
                                //send_json(&mut writer, &res).await.unwrap();
                                if let Err(e) = send_json(&mut writer, &res).await {
                                    eprintln!("Error sending response: {}", e);
                                    break;
                                }
                                {
                                    let mut manager = metadata.lock().await;
                                    manager.remove(&username, &filename).await;
                                }
                            }
                            Err(_) => {
                                let res = Response::Error {
                                    message: "File not found".to_string(),
                                };
                                //let res_j = serde_json::to_vec(&res).unwrap();
                                //writer.write_all(&res_j).await.unwrap();
                                //writer.flush().await.unwrap();
                                if let Err(e) =send_json(&mut writer, &res).await{
                                    eprintln!("Error sending response: {}",e);
                                    break;
                                }
                            }
                        }
                    }
                    Request::Rename {
                        session_id,
                        old_name,
                        new_name,
                    } => {
                        {
                            let manager = sessions.lock().await;
                            if !manager.validate(session_id.clone()) {
                                let response = Response::Error {
                                    message: "Invalid session".to_string(),
                                };
                                //let res_j = serde_json::to_vec(&response).unwrap();
                                //writer.write_all(&res_j).await.unwrap();
                                //writer.flush().await.unwrap();
                                //send_json(&mut writer, &response).await.unwrap();
                                if let Err(e) = send_json(&mut writer, &response).await {
                                    eprintln!("Error sending response: {}", e);
                                    break;
                                }
                                continue;
                            }
                        }
                        let username = {
                            let manager = sessions.lock().await;
                            manager.get_username(session_id.clone()).unwrap()
                        };
                        let old_path = format!("storage/{}/{}", username.clone(), old_name.clone());
                        let new_path = format!("storage/{}/{}", username.clone(), new_name.clone());
                        match tokio::fs::rename(&old_path, &new_path).await {
                            Ok(_) => {
                                let response = Response::Success {
                                    message: "File renamed successfully".to_string(),
                                };
                                //let res_j = serde_json::to_vec(&response).unwrap();
                                //writer.write_all(&res_j).await.unwrap();
                                //writer.flush().await.unwrap();
                                //send_json(&mut writer, &response).await.unwrap();
                                if let Err(e) = send_json(&mut writer, &response).await {
                                    eprintln!("Error sending response: {}", e);
                                    break;
                                }
                                {
                                    let mut manager = metadata.lock().await;
                                    manager.rename(&username, &old_name, &new_name).await;
                                }
                            }
                            Err(_) => {
                                let response = Response::Error {
                                    message: "Failed to rename file".to_string(),
                                };
                                //let res_j = serde_json::to_vec(&response).unwrap();
                                //writer.write_all(&res_j).await.unwrap();
                                //writer.flush().await.unwrap();
                                //send_json(&mut writer, &response).await.unwrap();
                                if let Err(e) = send_json(&mut writer, &response).await {
                                    eprintln!("Error sending response: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Request::Logout { session_id } => {
                        let response = {
                            let mut manager = sessions.lock().await;
                            if manager.remove_session(session_id) {
                                Response::Success {
                                    message: "Logout successful".to_string(),
                                }
                            } else {
                                Response::Error {
                                    message: "Invalid session ID".to_string(),
                                }
                                //this couldnt happen
                            }
                        };
                        //let res_j = serde_json::to_vec(&response).unwrap();
                        //writer.write_all(&res_j).await.unwrap();
                        //writer.flush().await.unwrap();
                        //send_json(&mut writer, &response).await.unwrap();
                        if let Err(e) = send_json(&mut writer, &response).await {
                            eprintln!("Error sending response: {}", e);
                            break;
                        }
                    }
                    Request::Search { session_id, query } => {
                        {
                            let manager = sessions.lock().await;
                            if !manager.validate(session_id.clone()) {
                                let response = Response::Error {
                                    message: "Invalid session ID".to_string(),
                                };
                                if let Err(e) = send_json(&mut writer, &response).await {
                                    eprintln!("Error sending response: {}", e);
                                    break;
                                }
                                continue;
                            }
                        }
                        let username = {
                            let manager = sessions.lock().await;
                            manager.get_username(session_id.clone()).unwrap()
                        };
                        let result = {
                            let manager = metadata.lock().await;
                            manager.search(&query, &username)
                        };
                        let response = Response::SearchResults { files: result };
                        if let Err(e) = send_json(&mut writer, &response).await {
                            eprintln!("Error sending response: {}", e);
                            break;
                        }
                    }
                    Request::ChangeAccess {
                        session_id,
                        filename,
                        access,
                    } => {
                        {
                            let manager = sessions.lock().await;
                            if !manager.validate(session_id.clone()) {
                                let response = Response::Error {
                                    message: "Invalid session ID".to_string(),
                                };
                                if let Err(e) =send_json(&mut writer, &response).await{
                                    eprintln!("Error sending response: {}",e);
                                    break;
                                }
                                continue;
                            }
                        }
                        let username = {
                            let manager = sessions.lock().await;
                            manager.get_username(session_id.clone()).unwrap()
                        };
                        {
                            let mut manager = metadata.lock().await;

                            let access_int = match access {
                                Access::Public => 1,
                                Access::Private => 2,
                            };
                            let file = manager.find(&username, &filename);
                            match file {
                                Some(file_find) => {
                                    let acccsess_file = match file_find.access {
                                        Access::Public => 1,
                                        Access::Private => 2,
                                    };
                                    if acccsess_file == access_int {
                                        let res = Response::Error {
                                            message: "its alearedy Private or public".to_string(),
                                        };
                                        if let Err(e) =send_json(&mut writer, &res).await{
                                            eprintln!("Error sending response: {}", e);
                                            break;
                                        }
                                        continue;
                                    }
                                    manager.update_access(&filename, &username, access).await;
                                    let res = Response::Success {
                                        message: "access changed".to_string(),
                                    };
                                    if let Err(e) = send_json(&mut writer, &res).await {
                                        eprintln!("Error sending response: {}", e);
                                        break;
                                    }
                                }
                                None => {
                                    let res = Response::Error {
                                        message: "file not found".to_string(),
                                    };
                                    if let Err(e) =send_json(&mut writer, &res).await{
                                        eprintln!("Error sending response: {}",e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Request::DeletSession { session_id } => {
                        let mut manager = sessions.lock().await;
                        manager.remove_session(session_id);
                    }
                }
            }
            println!("Connection closed: {}", address);
            // Handle the connection in a separate task
            // ...
        });
    }
}
