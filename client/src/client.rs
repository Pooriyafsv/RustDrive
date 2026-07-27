use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use common::protocol::{Request, Response};
use tokio::fs::{self, File};
use tokio::io::BufReader;
use common::protocol_io::{recv_json,send_json};

pub async fn start_client() {
    println!("{:?}",std::env::current_dir().unwrap());
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    println!("Connected to server");
    let (mut socket_read, mut socket_write) = stream.split();
    let mut socket_read = BufReader::new(socket_read);
    
    let request = Request::Ping;
    //let data = serde_json::to_vec(&request).unwrap();
    //socket_write.write_all(&data).await.unwrap();
    send_json(&mut socket_write, &request).await.unwrap();
    
    //let mut buf = [0; 1024];
    //let n = socket_read.read(&mut buf).await.unwrap();
    
    //let res = serde_json::from_slice::<Response>(&buf[..n]);
    let res = recv_json(&mut socket_read).await;
    match res {
        Ok(response) => {
            match response {
                Response::Pong => {
                    // متغیرها باید بیرون حلقه تعریف شوند تا وضعیت لاگین حفظ شود
                    let mut user_session_id = String::new();
                    let mut is_login = false;
                    
                    loop {
                        if !is_login {
                            println!("1. Login\n2. Register\n>");
                            let mut input = String::new();
                            std::io::stdin().read_line(&mut input).unwrap();
                            
                            match input.trim().parse::<u8>() {
                                Ok(input_num) => {
                                    match input_num {
                                        1 => {
                                            println!("Enter your username\n >");
                                            let mut username = String::new();
                                            std::io::stdin().read_line(&mut username).unwrap();
                                            
                                            println!("Enter your password\n  >");
                                            let mut password = String::new();
                                            std::io::stdin().read_line(&mut password).unwrap();
                                            
                                            let request = Request::Login {
                                                username: username.trim().to_string(),
                                                password: password.trim().to_string()
                                            };
                                            //let data = serde_json::to_vec(&request).unwrap();
                                            //socket_write.write_all(&data).await.unwrap();
                                            send_json(&mut socket_write, &request).await.unwrap();
                                            
                                            //let mut buffer = [0; 1024];
                                            //let n = socket_read.read(&mut buffer).await.unwrap();
                                            match recv_json(&mut socket_read).await {
                                                Ok(response) => {
                                                    match response {
                                                        Response::LoginSuccess { session_id } => {
                                                            println!("Login successful");
                                                            user_session_id = session_id.clone();
                                                            is_login = true;
                                                        }
                                                        Response::Error { message } => {
                                                            eprintln!("Error: {}", message);
                                                        }
                                                        _ => eprintln!("Unexpected response"),
                                                    }
                                                }
                                                Err(err) => {
                                                    eprintln!("Error: {}", err);
                                                }
                                            }
                                        }
                                        2 => {
                                            println!("please enter your name:");
                                            let mut name = String::new();
                                            std::io::stdin().read_line(&mut name).unwrap();
                                            let name = name.trim().to_string();
                                            
                                            println!("password:");
                                            let mut password = String::new();
                                            std::io::stdin().read_line(&mut password).unwrap();
                                            let password = password.trim().to_string();
                                            
                                            let request = Request::Register { username: name, password };
                                            //let data = serde_json::to_vec(&request).unwrap();
                                            //socket_write.write_all(&data).await.unwrap();
                                            send_json(&mut socket_write, &request).await.unwrap();
                                            
                                            //let mut buf = [0; 1024];
                                            //let n = socket_read.read(&mut buf).await.unwrap();
                                            //let res = serde_json::from_slice::<Response>(&buf[..n]);
                                            let res = recv_json(&mut socket_read).await;
                                            match res {
                                                Ok(response) => {
                                                    match response {
                                                        Response::Success { message } => {
                                                            println!("Register status: {}", message);
                                                        }
                                                        Response::Error { message } => {
                                                            println!("Error: {}", message);
                                                        }
                                                        _ => println!("Unexpected response"),
                                                    }
                                                }
                                                Err(e) => println!("Error: {}", e),
                                            }
                                        }
                                        _ => eprintln!("wrong input"),
                                    }
                                }
                                Err(e) => eprintln!("Error: {}", e),
                            }
                        } // پایان if !is_login
                        
                        if is_login {
                            println!("1.Upload\n 2.Download\n 3.FileList\n 4.Delete\n 5.Exit");
                            let mut input = String::new();
                            std::io::stdin().read_line(&mut input).unwrap(); // اضافه شدن unwrap
                            
                            match input.trim().parse::<u16>() {
                                Ok(1) => {
                                    println!("Enter file name: ");
                                    let mut input = String::new();
                                    std::io::stdin().read_line(&mut input).unwrap();

                                    let file_name = input.trim();

                                    if fs::metadata(file_name).await.is_err() {
                                        eprintln!("File not found");
                                    } else {
                                        let metadata = fs::metadata(file_name).await.unwrap();

                                        if !metadata.is_file() {
                                            eprintln!("This is not a file.");
                                            continue;
                                        }

                                        let file_size = metadata.len();

                                        let req = Request::Upload {
                                            session_id: user_session_id.clone(),
                                            filename: file_name.to_string(),
                                            size: file_size,
                                        };

                                        // ارسال درخواست
                                        send_json(&mut socket_write, &req).await.unwrap();

                                        // دریافت ReadyForUpload
                                        let response: Response = recv_json(&mut socket_read).await.unwrap();

                                        match response {
                                            Response::ReadyForUpload => {
                                                let mut file = File::open(file_name).await.unwrap();
                                                let mut buffer = [0u8; 4096];

                                                loop {
                                                    let n = file.read(&mut buffer).await.unwrap();

                                                    if n == 0 {
                                                        break;
                                                    }

                                                    socket_write.write_all(&buffer[..n]).await.unwrap();
                                                }

                                                // دریافت Success
                                                let response: Response = recv_json(&mut socket_read).await.unwrap();

                                                match response {
                                                    Response::Success { message } => {
                                                        println!("File uploaded successfully: {}", message);
                                                    }

                                                    Response::Error { message } => {
                                                        eprintln!("Upload failed: {}", message);
                                                    }

                                                    _ => {
                                                        eprintln!("Unexpected response");
                                                    }
                                                }
                                            }

                                            Response::Error { message } => {
                                                eprintln!("{}", message);
                                            }

                                            _ => {
                                                eprintln!("Unexpected response");
                                            }
                                        }
                                    }
                                }
                                Ok(2) => {
                                    let mut file_name = String::new();
                                    println!("Enter file name: ");
                                    std::io::stdin().read_line(&mut file_name).unwrap();
                                    let file_name = file_name.trim();
                                    let req = Request::Download { session_id:user_session_id.clone(), filename: file_name.to_string() };
                                    //let req_j = serde_json::to_vec(&req).unwrap();
                                    //socket_write.write_all(&req_j).await.unwrap();
                                    send_json(&mut socket_write, &req).await.unwrap();
                                    //let mut buff = [0u8; 1024];
                                    //let n = socket_read.read(&mut buff).await.unwrap();
                                    //let resp = serde_json::from_slice::<Response>(&buff[..n]).unwrap();
                                    let resp: Response = recv_json(&mut socket_read).await.unwrap();
                                    match resp {
                                        Response::ReadyForDownload { size } => {
                                            let mut file = File::create(file_name).await.unwrap();
                                            let mut downloaded = 0;
                                            while downloaded < size {
                                                let mut buff = [0u8; 4096];
                                                let n = socket_read.read(&mut buff).await.unwrap();
                                                file.write_all(&buff[..n]).await.unwrap();
                                                downloaded += n as u64;
                                            }
                                            println!("Download complete");

                                        }
                                        Response::Error { message } =>{
                                            eprintln!("Error:{}",message);
                                        }
                                        _ =>{
                                            eprintln!("Unexpected response");
                                        }
                                    }
                                    
                                }
                                Ok(3 ) =>{
                                    let req = Request::List { session_id: user_session_id.clone() };
                                    //let req_j = serde_json::to_vec(&req).unwrap();
                                    //socket_write.write_all(&req_j).await.unwrap();
                                    send_json(&mut socket_write, &req).await.unwrap();
                                    //let mut buff = [0u8;2048];
                                    //let n = socket_read.read(& mut buff).await.unwrap();
                                    //let res = serde_json::from_slice::<Response>(&buff[..n]).unwrap();
                                    let res = recv_json(&mut socket_read).await.unwrap();
                                    match res {
                                        Response::FileList { files } =>{
                                            for (i,name) in files.iter().enumerate(){
                                                println!("{}.{}",i+1,name);
                                            }
                                        }
                                        Response::Error { message } =>{
                                            eprintln!("Error:{}",message);
                                        }
                                        _ => eprintln!("Unknown response"),
                                    }
                                }
                                Ok(4) => {
                                    println!("Enter file name:");
                                    let mut file_name = String::new();
                                    std::io::stdin().read_line(&mut file_name).unwrap();
                                    let file_name = file_name.trim();
                                    let req = Request::Delete { session_id: user_session_id.clone(), filename: file_name.to_string() };
                                    //let req_j = serde_json::to_vec(&req).unwrap();
                                    //socket_write.write_all(&req_j).await.unwrap();
                                    send_json(&mut socket_write, &req).await.unwrap();
                                    //let mut buff = [0u8;2048];
                                    //let res = socket_read.read(&mut buff).await.unwrap();
                                    //let res_j = serde_json::from_slice::<Response>(&buff[..res]).unwrap();
                                    let res_j = recv_json(&mut socket_read).await.unwrap();
                                    match res_j {
                                        Response::Success { message } => println!("{}", message),
                                        Response::Error { message } => eprintln!("{}", message),
                                        _ => eprintln!("Unknown response"),
                                    }
                                }
                                Ok(5) => {
                                    println!("please enter file name");
                                    let mut file_name = String::new();
                                    std::io::stdin().read_line(&mut file_name).unwrap();
                                    let  old_name = file_name.clone().trim().to_string();
                                    let mut new_name = String::new();
                                    println!("please enter new file name");
                                    std::io::stdin().read_line(&mut new_name).unwrap();
                                    let new_name = new_name.trim().to_string();
                                    let req = Request::Rename { session_id:user_session_id.clone(),old_name:old_name, new_name:new_name };
                                    //let req_j = serde_json::to_vec(&req).unwrap();
                                    //socket_write.write_all(&req_j).await.unwrap();
                                    send_json(&mut socket_write, &req).await.unwrap();
                                    //let mut buff = [0u8; 1024];
                                    //let n = socket_read.read(&mut buff).await.unwrap();
                                    //let res: Response = serde_json::from_slice(&buff[..n]).unwrap();
                                    let res = recv_json(&mut socket_read).await.unwrap();
                                    match res {
                                        Response::Success { message } => {
                                            println!("{}", message);
                                        }
                                        Response::Error { message } => {
                                            eprintln!("{}",message);
                                        }
                                        _ => eprintln!("Unknown response"),
                                    }
                                }
                                Ok(6) => {
                                    let req = Request::Logout{session_id:user_session_id.clone()};
                                    //let req_j = serde_json::to_vec(&req).unwrap();
                                    //socket_write.write_all(&req_j).await.unwrap();
                                    send_json(&mut socket_write, &req).await.unwrap();
                                    //let mut buff = [0u8; 1024];
                                    //let n = socket_read.read(&mut buff).await.unwrap();
                                    //let res: Response = serde_json::from_slice(&buff[..n]).unwrap();
                                    let res = recv_json(&mut socket_read).await.unwrap();
                                    match res {
                                        Response::Success { message } => {
                                            println!("{}",message);
                                            user_session_id.clear();
                                            is_login = false;
                                        }
                                        Response::Error { message } => {
                                            println!("{}",message);
                                            user_session_id.clear();
                                            is_login = false;
                                        }
                                        _ => eprintln!("wrong responset"),
                                    }
                                }


                                _ => eprintln!("wrong input"),
                            }
                        }
                    } // پایان loop
                }
                _ => eprintln!("Unexpected response"),
            }
        }
        Err(e) => println!("Unexpected response: {}", e),
    }
}