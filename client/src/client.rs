use crate::progress::print_progress;
use common::models::Access::{Private, Public};
use common::protocol::{Request, Response};
use common::protocol_io::{recv_json, send_json};
use tokio::fs::{self, File};
use tokio::io::BufReader;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn start_client() {
    //println!("{:?}", std::env::current_dir().unwrap());
    let  stream = TcpStream::connect("127.0.0.1:8080").await;
    let mut stream = match stream {
        Ok(stream) => stream,
        Err(_) => {
            println!("Failed to connect to server");
            return;
        }
    };
    println!("Connected to server");
    let ( socket_read, mut socket_write) = stream.split();
    let mut socket_read = BufReader::new(socket_read);

    let request = Request::Ping;
    //let data = serde_json::to_vec(&request).unwrap();
    //socket_write.write_all(&data).await.unwrap();
    if let Err(e)=send_json(&mut socket_write, &request).await{
        println!("Failed to send request: {}", e);
        return;
    }

    //let mut buf = [0; 1024];
    //let n = socket_read.read(&mut buf).await.unwrap();

    //let res = serde_json::from_slice::<Response>(&buf[..n]);
    let res = recv_json(&mut socket_read).await;
    match res {
        Ok(response) => {
            match response {
                Response::Pong => {
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
                                                password: password.trim().to_string(),
                                            };
                                            //let data = serde_json::to_vec(&request).unwrap();
                                            //socket_write.write_all(&data).await.unwrap();
                                            if let Err(e)=send_json(&mut socket_write, &request).await{
                                                eprintln!("Error sending request: {}", e);
                                                break;
                                            }

                                            //let mut buffer = [0; 1024];
                                            //let n = socket_read.read(&mut buffer).await.unwrap();
                                            match recv_json(&mut socket_read).await {
                                                Ok(response) => match response {
                                                    Response::LoginSuccess { session_id } => {
                                                        println!("Login successful");
                                                        user_session_id = session_id.clone();
                                                        is_login = true;
                                                    }
                                                    Response::Error { message } => {
                                                        eprintln!("Error: {}", message);
                                                    }
                                                    _ => eprintln!("Unexpected response"),
                                                },
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

                                            let request = Request::Register {
                                                username: name,
                                                password,
                                            };
                                            //let data = serde_json::to_vec(&request).unwrap();
                                            //socket_write.write_all(&data).await.unwrap();
                                            if let Err(e)=send_json(&mut socket_write, &request).await{
                                                eprintln!("Error sending request: {}",e);
                                                return;
                                            }

                                            //let mut buf = [0; 1024];
                                            //let n = socket_read.read(&mut buf).await.unwrap();
                                            //let res = serde_json::from_slice::<Response>(&buf[..n]);
                                            let res = recv_json(&mut socket_read).await;
                                            match res {
                                                Ok(response) => match response {
                                                    Response::Success { message } => {
                                                        println!("Register status: {}", message);
                                                    }
                                                    Response::Error { message } => {
                                                        println!("Error: {}", message);
                                                    }
                                                    _ => println!("Unexpected response"),
                                                },
                                                Err(e) => println!("Error: {}", e),
                                            }
                                        }
                                        _ => eprintln!("wrong input"),
                                    }
                                }
                                Err(e) => eprintln!("Error: {}", e),
                            }
                        }

                        if is_login {
                            println!("1.Upload\n2.Download\n3.FileList\n4.Delete\n5 Rename\n6 Logout\n7 Search\n8 Change Access\n9 Exit");
                            let mut input = String::new();
                            std::io::stdin().read_line(&mut input).unwrap(); 

                            match input.trim().parse::<u16>() {
                                Ok(1) => {
                                    println!("Enter file name: ");
                                    let mut input = String::new();
                                    std::io::stdin().read_line(&mut input).unwrap();

                                    let file_name = input.trim();
                                    println!("Looking for file: '{}'", file_name);
                                    println!(
                                        "Current directory: {:?}",
                                        std::env::current_dir().unwrap()
                                    );
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

                                        
                                        if let Err(e)=send_json(&mut socket_write, &req).await{
                                            eprintln!("Error sending request: {}", e);
                                            break;
                                        }

                                        
                                        let response: Response =
                                            match recv_json(&mut socket_read).await{
                                                Ok(response) => response,
                                                Err(e) => {
                                                    eprintln!("Error receiving response: {}", e);
                                                    break;
                                                }
                                            };

                                        match response {
                                            Response::ReadyForUpload => {
                                                let mut file = File::open(file_name).await.unwrap();
                                                let mut buffer = [0u8; 4096];
                                                let mut cur = 0;
                                                loop {
                                                    let n = file.read(&mut buffer).await.unwrap();

                                                    cur += n;
                                                    
                                                    if n == 0 {
                                                        break;
                                                    }
                                                    print_progress(cur as u64, file_size);
                                                    socket_write
                                                        .write_all(&buffer[..n])
                                                        .await
                                                        .unwrap();
                                                }

                                                // دریافت Success
                                                let response: Response =
                                                    match recv_json(&mut socket_read).await{
                                                    Ok(response) => response,
                                                    Err(_) => {
                                                        println!("Error receiving response");
                                                        continue;
                                                    }
                                                    };

                                                match response {
                                                    Response::Success { message } => {
                                                        println!(
                                                            "File uploaded successfully: {}",
                                                            message
                                                        );
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
                                    println!(
                                        "Enter owner user name:(if is your file enter your name)"
                                    );
                                    let mut owner = String::new();
                                    std::io::stdin().read_line(&mut owner).unwrap();
                                    let owner = owner.trim().to_string();
                                    let req = Request::Download {
                                        session_id: user_session_id.clone(),
                                        filename: file_name.to_string(),
                                        owner: owner,
                                    };
                                    //let req_j = serde_json::to_vec(&req).unwrap();
                                    //socket_write.write_all(&req_j).await.unwrap();
                                    if let Err(e)=send_json(&mut socket_write, &req).await{
                                        eprintln!("Error sending request: {}",e);
                                        break;
                                    }
                                    //let mut buff = [0u8; 1024];
                                    //let n = socket_read.read(&mut buff).await.unwrap();
                                    //let resp = serde_json::from_slice::<Response>(&buff[..n]).unwrap();
                                    let resp: Response = match recv_json(&mut socket_read).await{
                                        Ok(resp) => resp,
                                        Err(e) => {
                                            eprintln!("Error receiving response: {}",e);
                                            break;
                                        }
                                    };
                                    match resp {
                                        Response::ReadyForDownload { size } => {
                                            let mut counter =1;
                                            let mut download_name = file_name.to_string();
                                            while fs::metadata(&download_name).await.is_ok(){
                                                download_name = format!("{}({})",file_name,counter);
                                                counter += 1;
                                            }
                                            let mut file = File::create(file_name).await.unwrap();
                                            let mut downloaded = 0;
                                            while downloaded < size {
                                                let mut buff = [0u8; 4096];
                                                let n = socket_read.read(&mut buff).await.unwrap();
                                                print_progress(downloaded as u64, size);
                                                file.write_all(&buff[..n]).await.unwrap();
                                                downloaded += n as u64;
                                            }
                                            println!("Download complete");
                                        }
                                        Response::Error { message } => {
                                            eprintln!("Error:{}", message);
                                        }
                                        _ => {
                                            eprintln!("Unexpected response");
                                        }
                                    }
                                }
                                Ok(3) => {
                                    let req = Request::List {
                                        session_id: user_session_id.clone(),
                                    };
                                    //let req_j = serde_json::to_vec(&req).unwrap();
                                    //socket_write.write_all(&req_j).await.unwrap();
                                    if let Err(e) =send_json(&mut socket_write, &req).await{
                                        eprintln!("Error:{}",e);
                                        break;
                                    }
                                    //let mut buff = [0u8;2048];
                                    //let n = socket_read.read(& mut buff).await.unwrap();
                                    //let res = serde_json::from_slice::<Response>(&buff[..n]).unwrap();
                                    let res = match recv_json(&mut socket_read).await{
                                        Ok(res) => res,
                                        Err(e) => {
                                            eprintln!("Error:{}",e);
                                            return;
                                        }
                                    };
                                    match res {
                                        Response::FileList { files } => {
                                            for (i, file) in files.iter().enumerate() {
                                                println!(
                                                    "{}.{} owner {} size {} created at {} category:{}",
                                                    i + 1,
                                                    file.filename,
                                                    file.owner,
                                                    file.filesize,
                                                    file.created_at,
                                                    file.category
                                                );
                                            }
                                        }
                                        Response::Error { message } => {
                                            eprintln!("Error:{}", message);
                                        }
                                        _ => eprintln!("Unknown response"),
                                    }
                                }
                                Ok(4) => {
                                    println!("Enter file name:");
                                    let mut file_name = String::new();
                                    std::io::stdin().read_line(&mut file_name).unwrap();
                                    let file_name = file_name.trim();
                                    let req = Request::Delete {
                                        session_id: user_session_id.clone(),
                                        filename: file_name.to_string(),
                                    };
                                    //let req_j = serde_json::to_vec(&req).unwrap();
                                    //socket_write.write_all(&req_j).await.unwrap();
                                    if let Err(e) = send_json(&mut socket_write, &req).await{
                                        eprintln!("Error sending request: {}",e);
                                        break;
                                    }
                                    //let mut buff = [0u8;2048];
                                    //let res = socket_read.read(&mut buff).await.unwrap();
                                    //let res_j = serde_json::from_slice::<Response>(&buff[..res]).unwrap();
                                    let res_j = match recv_json(&mut socket_read).await{
                                        Ok(res) => res,
                                        Err(e) => {
                                            eprintln!("Error receiving response: {}",e);
                                            break;
                                        }
                                    };
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
                                    let old_name = file_name.clone().trim().to_string();
                                    let mut new_name = String::new();
                                    println!("please enter new file name");
                                    std::io::stdin().read_line(&mut new_name).unwrap();
                                    let new_name = new_name.trim().to_string();
                                    let req = Request::Rename {
                                        session_id: user_session_id.clone(),
                                        old_name: old_name,
                                        new_name: new_name,
                                    };
                                    //let req_j = serde_json::to_vec(&req).unwrap();
                                    //socket_write.write_all(&req_j).await.unwrap();
                                    if let Err(e) =send_json(&mut socket_write, &req).await{
                                        eprintln!("error sending request: {}",e);
                                        break;
                                    }
                                    //let mut buff = [0u8; 1024];
                                    //let n = socket_read.read(&mut buff).await.unwrap();
                                    //let res: Response = serde_json::from_slice(&buff[..n]).unwrap();
                                    let res = match recv_json(&mut socket_read).await{
                                        Ok(res) => res,
                                        Err(e) => {
                                            eprintln!("error receiving response: {}",e);
                                            break;
                                        }
                                    };
                                    match res {
                                        Response::Success { message } => {
                                            println!("{}", message);
                                        }
                                        Response::Error { message } => {
                                            eprintln!("{}", message);
                                        }
                                        _ => eprintln!("Unknown response"),
                                    }
                                }
                                Ok(6) => {
                                    let req = Request::Logout {
                                        session_id: user_session_id.clone(),
                                    };
                                    //let req_j = serde_json::to_vec(&req).unwrap();
                                    //socket_write.write_all(&req_j).await.unwrap();
                                    if let Err(e) = send_json(&mut socket_write, &req).await{
                                        eprintln!("Error sending request: {}",e);
                                        break;
                                    }
                                    //let mut buff = [0u8; 1024];
                                    //let n = socket_read.read(&mut buff).await.unwrap();
                                    //let res: Response = serde_json::from_slice(&buff[..n]).unwrap();
                                    let res = match recv_json(&mut socket_read).await{
                                        Ok(res) => res,
                                        Err(e) => {
                                            eprintln!("Error receiving response: {}",e);
                                            break;
                                        }
                                    };
                                    match res {
                                        Response::Success { message } => {
                                            println!("{}", message);
                                            user_session_id.clear();
                                            is_login = false;
                                        }
                                        Response::Error { message } => {
                                            println!("{}", message);
                                            user_session_id.clear();
                                            is_login = false;
                                        }
                                        _ => eprintln!("wrong responset"),
                                    }
                                }
                                Ok(7) => {
                                    println!("search:");
                                    let mut query = String::new();
                                    std::io::stdin().read_line(&mut query).unwrap();
                                    let req = Request::Search {
                                        query: query.trim().to_string(),
                                        session_id: user_session_id.clone(),
                                    };
                                    if let Err(e) = send_json(&mut socket_write, &req).await{
                                        eprintln!("error: {}",e);
                                        break;
                                    }
                                    let res = match recv_json(&mut socket_read).await{
                                        Ok(res) => res,
                                        Err(e) => {
                                            eprintln!("error: {}",e);
                                            break;
                                        }
                                    };
                                    match res {
                                        Response::SearchResults { files } => {
                                            if files.len() == 0 {
                                                println!("no files found");
                                                continue;
                                            }
                                            for (i, file) in files.into_iter().enumerate() {
                                                println!(
                                                    "{}.{} created by{} at{} category:{}",
                                                    i,
                                                    file.filename,
                                                    file.owner,
                                                    file.created_at,
                                                    file.category
                                                );
                                            }
                                        }
                                        Response::Error { message } => eprintln!("{}", message),
                                        _ => eprintln!("wrong response"),
                                    }
                                }
                                Ok(8) => {
                                    println!("enter file name:");
                                    let mut filename = String::new();
                                    std::io::stdin()
                                        .read_line(&mut filename)
                                        .expect("failed to read line");
                                    let filename = filename.trim();
                                    println!("choose your Option:\n1.public\n2.Private");
                                    let mut option = String::new();
                                    std::io::stdin()
                                        .read_line(&mut option)
                                        .expect("failed to read line");
                                    let option = option.trim();
                                    match option {
                                        "1" => {
                                            let req = Request::ChangeAccess {
                                                session_id: user_session_id.clone(),
                                                filename: filename.to_string(),
                                                access: Public,
                                            };
                                            if let Err(e) = send_json(&mut socket_write, &req).await{
                                                eprintln!("error sending request: {}",e);
                                                continue;
                                            }
                                            let res = match recv_json(&mut socket_read).await{
                                                Ok(res) => res,
                                                Err(e) => {
                                                    eprintln!("error receiving response: {}",e);
                                                    continue;
                                                }
                                            };
                                            match res {
                                                Response::Success { message } => {
                                                    println!("{}", message);
                                                }
                                                Response::Error { message } => {
                                                    eprintln!("Error: {}", message);
                                                }
                                                _ => eprintln!("try again"),
                                            }
                                        }
                                        "2" => {
                                            let req = Request::ChangeAccess {
                                                session_id: user_session_id.clone(),
                                                filename: filename.to_string(),
                                                access: Private,
                                            };
                                            //send_json(&mut socket_write, &req).await.unwrap();
                                            //let res = recv_json(&mut socket_read).await.unwrap();
                                            if let Err(e) = send_json(&mut socket_write, &req).await{
                                                eprintln!("error sending request: {}",e);
                                                continue;
                                            }
                                            let res = match recv_json(&mut socket_read).await{
                                                Ok(res) => res,
                                                Err(e) => {
                                                    eprintln!("error receiving response: {}",e);
                                                    continue;
                                                }
                                            };
                                            match res {
                                                Response::Success { message } => {
                                                    println!("{}", message);
                                                }
                                                Response::Error { message } => {
                                                    eprintln!("Error: {}", message);
                                                }
                                                _ => eprintln!("try again"),
                                            }
                                        }
                                        _ => eprintln!("wrong input"),
                                    }
                                }
                                Ok(9) => {
                                    println!("Goodbye!");
                                    let req = Request::DeletSession { session_id: user_session_id };
                                    if let Err(e) = send_json(&mut socket_write, &req).await {
                                        eprintln!("Error: {}", e);
                                    }
                                    break;
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
