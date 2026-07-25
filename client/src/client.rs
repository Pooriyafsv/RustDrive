use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use common::protocol::{Request, Response};


pub async fn start_client(){
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    println!("Connected to server");
    let (mut socket_read, mut socket_write) = stream.split();
    let request = Request::Ping;
    let data = serde_json::to_vec(&request).unwrap();
    socket_write.write_all(&data).await.unwrap();
    let mut buf = [0; 1024];
    let n = socket_read.read(&mut buf).await.unwrap();
    //println!("Received {} bytes",String::from_utf8_lossy(&buf[..n]));
    let res = serde_json::from_slice::<Response>(&buf[..n]);
    match res {
        Ok(Response) => println!("{:?}",Response),
        Err(e) => println!("Unexpected response{}",e),
    }
    println!("please enter your name:");
    let mut name = String::new();
    std::io::stdin().read_line(&mut name).unwrap();
    println!("password:");
    name = name.trim().to_string();
    let mut password = String::new();
    std::io::stdin().read_line(&mut password).unwrap();
    password = password.trim().to_string();
    let request = Request::Register {username : name,password : password};
    let data = serde_json::to_vec(&request).unwrap();
    socket_write.write_all(&data).await.unwrap();
    let mut buf = [0; 1024];
    let n = socket_read.read(&mut buf).await.unwrap();
    let res = serde_json::from_slice::<Response>(&buf[..n]);
    match res {
        Ok(Response) => println!("{:?}",Response),
        Err(e) => println!("Unexpected response{}",e),
    }

    
    // stream.read(&mut buf).await.unwrap();
    // println!("{}", String::from_utf8_lossy(&buf));
}