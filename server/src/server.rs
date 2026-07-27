use tokio::fs::File;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt,BufReader};
use common::protocol::{Request, Response};
use common::models::{User};
use crate::auth;
use crate::sessionmanager::SessionManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs;
use common::protocol_io::{send_json,recv_json};
pub async fn start_server(
    sessions:Arc<Mutex<SessionManager>>,
) {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server started on port 8080");

    loop {
        let (mut socket, address) = listener.accept().await.unwrap();
        let sessions = sessions.clone();
        tokio::spawn(async move {
            println!("New connection: {}", address);
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            loop {
                let data: Request = recv_json(&mut reader).await.unwrap();
                match data {
                    Request::Ping => {
                        let res = Response::Pong;
                        //let res_j = serde_json::to_vec(&res).unwrap();
                        //println!("Sending response: {:?}",res_j);
                        send_json(&mut writer, &res).await.unwrap();
                    },
                    Request::Register { username, password } => {
                        let user = User { username:username, password:password };
                        let response = auth::register(user);
                        send_json(&mut writer, &response).await.unwrap();
                    }
                    Request::Login { username, password } => {
                        let user = User { username:username, password:password };
                        let response = match auth::login(user) {
                            Ok(user) => {
                                let mut manager = sessions.lock().await;
                                let session_id = manager.create_session(user.username);
                                Response::LoginSuccess { session_id }
                            }
                            Err(e) => {
                                Response::Error { message:e }
                            }
                        };
                        //let res_j = serde_json::to_vec(&response).unwrap();
                        //writer.write_all(&res_j).await.unwrap();
                        //writer.flush().await.unwrap();
                        send_json(&mut writer, &response).await.unwrap();
                    }
                    Request::Upload { session_id, filename, size } => {
                        {let manager = sessions.lock().await;
                        if !manager.validate(session_id.clone()) {
                            let response = Response::Error { message: "Invalid session".to_string() };
                            send_json(&mut writer, &response).await.unwrap();
                            continue;
                        }};
                        let username ={
                            let manager = sessions.lock().await;
                            manager.get_username(session_id.clone()).unwrap()
                        };
                        let dir = format!("storage/{}", username);
                        fs::create_dir_all(&dir).await.unwrap();
                        let path = format!("{}/{}", dir, filename);
                        let mut file = File::create(&path).await.unwrap();
                        let res = Response::ReadyForUpload;
                        //let res_j = serde_json::to_vec(&res).unwrap();
                        //writer.write_all(&res_j).await.unwrap();
                        //writer.flush().await.unwrap();
                        send_json(&mut writer, &res).await.unwrap();
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
                        let res = Response::Success { message: "upload complete".to_string() };
                        //let res_j = serde_json::to_vec(&res).unwrap();
                        //writer.write_all(&res_j).await.unwrap();
                        //writer.flush().await.unwrap();  
                        send_json(&mut writer, &res).await.unwrap();


                        // Handle the upload
                        // ...
                    }
                    Request::Download { session_id, filename } => {
                        {let manager = sessions.lock().await;
                        if !manager.validate(session_id.clone()){
                            let response = Response::Error { message: "Invalid session".to_string() };
                            //let res_j = serde_json::to_vec(&response).unwrap();
                            //writer.write_all(&res_j).await.unwrap();
                            //writer.flush().await.unwrap();
                            send_json(&mut writer, &response).await.unwrap();
                            continue;
                        }}
                        let username ={
                            let manager = sessions.lock().await;
                            manager.get_username(session_id.clone()).unwrap()
                        };
                        let path = format!("storage/{}/{}", username,filename);
                        if fs::metadata(&path).await.is_err() {
                            let res = Response::Error { message: "File not found".to_string() };
                            //let res_j = serde_json::to_vec(&res).unwrap();
                            //writer.write_all(&res_j).await.unwrap();
                            //writer.flush().await.unwrap();
                            send_json(&mut writer, &res).await.unwrap();
                            continue;   
                        }
                        let metadata = fs::metadata(&path).await.unwrap();
                        let size = metadata.len();
                        let res = Response::ReadyForDownload { size };
                        //let res_j= serde_json::to_vec(&res).unwrap();
                        //writer.write_all(&res_j).await.unwrap();
                        //writer.flush().await.unwrap();
                        send_json(&mut writer, &res).await.unwrap();
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
                    Request::List { session_id } =>{
                        {let manager = sessions.lock().await;
                        if !manager.validate(session_id.clone()){
                            let response = Response::Error { message: "Invalid session".to_string() };
                            //let res_j = serde_json::to_vec(&response).unwrap();
                            //writer.write_all(&res_j).await.unwrap();
                            //writer.flush().await.unwrap();
                            send_json(&mut writer, &response).await.unwrap();
                            continue;
                        }}
                        let username ={
                            let manager = sessions.lock().await;
                            manager.get_username(session_id.clone()).unwrap()
                        };
                        let path = format!("storage/{}",username);
                        if fs::metadata(&path).await.is_err() {
                            let response = Response::Error {
                            message: "you don't have a storage folder".to_string(),
                            };
                            //let res_j = serde_json::to_vec(&response).unwrap();
                            //writer.write_all(&res_j).await.unwrap();
                            //writer.flush().await.unwrap();
                            send_json(&mut writer, &response).await.unwrap();
                            
                            continue;
                        }
                        let mut dir = tokio::fs::read_dir(path).await.unwrap();
                        let mut files = Vec::new();
                        loop {
                            let entry = dir.next_entry().await.unwrap();
                            match entry {
                                Some(file) =>{
                                    let name = file.file_name().to_string_lossy().to_string();
                                    files.push(name);
                                }
                                None => break,
                            }
                        }
                        let res = Response::FileList { files };
                        //let res_j = serde_json::to_vec(&res).unwrap();
                        //writer.write_all(&res_j).await.unwrap();
                        //writer.flush().await.unwrap();
                        send_json(&mut writer, &res).await.unwrap();
                    }
                    Request::Delete { session_id, filename } =>{
                        {let manager = sessions.lock().await;
                        if !manager.validate(session_id.clone()){
                            let response = Response::Error { message: "Invalid session".to_string() };
                            //let res_j = serde_json::to_vec(&response).unwrap();
                            //writer.write_all(&res_j).await.unwrap();
                            //writer.flush().await.unwrap();
                            send_json(&mut writer, &response).await.unwrap();
                            continue;
                        }}
                        let username ={
                            let manager = sessions.lock().await;
                            manager.get_username(session_id.clone()).unwrap()
                        };
                        let path = format!("storage/{}/{}",username,filename);
                        match tokio::fs::remove_file(path).await {
                            Ok(_) =>{
                                let res = Response::Success { message: "File deleted succefuly".to_string() };
                                //let res_j = serde_json::to_vec(&res).unwrap();
                                //writer.write_all(&res_j).await.unwrap();
                                //writer.flush().await.unwrap();
                                send_json(&mut writer, &res).await.unwrap();

                            }
                            Err(_) =>{
                                let res = Response::Error { message: "File not found".to_string() };
                                //let res_j = serde_json::to_vec(&res).unwrap();
                                //writer.write_all(&res_j).await.unwrap();
                                //writer.flush().await.unwrap();
                                send_json(&mut writer, &res).await.unwrap();
                            }
                        }
                    }
                    Request::Rename { session_id, old_name, new_name } =>{
                        {let manager = sessions.lock().await;
                        if !manager.validate(session_id.clone()){
                            let response = Response::Error { message: "Invalid session".to_string() };
                            //let res_j = serde_json::to_vec(&response).unwrap();
                            //writer.write_all(&res_j).await.unwrap();
                            //writer.flush().await.unwrap();
                            send_json(&mut writer, &response).await.unwrap();
                            continue;
                        }}
                        let username ={
                            let manager = sessions.lock().await;
                            manager.get_username(session_id.clone()).unwrap()
                        };
                        let old_path = format!("storage/{}/{}", username, old_name);
                        let new_path = format!("storage/{}/{}", username, new_name);
                        match tokio::fs::rename(&old_path, &new_path).await{
                            Ok(_) => {
                                let response = Response::Success { message: "File renamed successfully".to_string() };
                                //let res_j = serde_json::to_vec(&response).unwrap();
                                //writer.write_all(&res_j).await.unwrap();
                                //writer.flush().await.unwrap();
                                send_json(&mut writer, &response).await.unwrap();
                            }
                            Err(_) => {
                                let response = Response::Error{ message: "Failed to rename file".to_string() };
                                //let res_j = serde_json::to_vec(&response).unwrap();
                                //writer.write_all(&res_j).await.unwrap();
                                //writer.flush().await.unwrap();
                                send_json(&mut writer, &response).await.unwrap();
                            }
                        }
                    }
                    Request::Logout { session_id } => {
                        let response ={
                            let mut manager = sessions.lock().await;
                            if manager.remove_session(session_id) {
                                Response::Success { message: "Logout successful".to_string() }
                            } else {
                                Response::Error { message: "Invalid session ID".to_string() }
                                //this couldnt happen
                            }
                        };
                        //let res_j = serde_json::to_vec(&response).unwrap();
                        //writer.write_all(&res_j).await.unwrap();
                        //writer.flush().await.unwrap();
                        send_json(&mut writer, &response).await.unwrap();
                    }

                    _ =>println!("Unknown request")
                }
            }
            println!("Connection closed: {}", address);
            // Handle the connection in a separate task
            // ...
        });
    }
}