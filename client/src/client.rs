use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn start_client(){
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    println!("Connected to server");
    let (mut socket_read, mut socket_write) = stream.split();
    socket_write.write_all("Hello, world!".as_bytes()).await.unwrap();
    let mut buf = [0; 1024];
    let n = socket_read.read(&mut buf).await.unwrap();
    println!("{}", String::from_utf8_lossy(&buf[..n]));
    // stream.read(&mut buf).await.unwrap();
    // println!("{}", String::from_utf8_lossy(&buf));
}