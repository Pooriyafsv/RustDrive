use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt,BufReader};
use common::protocol::{Request, Response};
use common::models::{User};
use crate::auth;
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
                let data = serde_json::from_slice::<Request>(&buff[..n]).unwrap();
                println!("Received request: {:?}", data);
                match data {
                    Request::Ping => {
                        let res = Response::Pong;
                        let res_j = serde_json::to_vec(&res).unwrap();
                        //println!("Sending response: {:?}",res_j);
                        writer.write_all(&res_j).await.unwrap();
                        writer.flush().await.unwrap();
                    },
                    Request::Register { username, password } => {
                        let user = User { username:username, password:password };
                        let response = auth::register(user);
                        let res_j = serde_json::to_vec(&response).unwrap();
                        writer.write_all(&res_j).await.unwrap();
                        writer.flush().await.unwrap();
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