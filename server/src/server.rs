use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt,BufReader};
pub async fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server started on port 8080");

    loop {
        let (mut socket, address) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            println!("New connection: {}", address);
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            loop {
                let mut buff = [0u8; 1024];
                let n = reader.read(&mut buff).await.unwrap();
                if n == 0 {
                    break;
                }
                println!("Received: {}", String::from_utf8_lossy(&buff[0..n]));
                let res = format!("welcome: {}", address);
                writer.write_all(res.as_bytes()).await.unwrap();
            }
            println!("Connection closed: {}", address);
            // Handle the connection in a separate task
            // ...
        });
    }
}