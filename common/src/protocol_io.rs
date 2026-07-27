use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

use serde::{Serialize, de::DeserializeOwned};

pub async fn send_json<W, T>(writer: &mut W, value: &T) -> std::io::Result<()>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let mut data = serde_json::to_vec(value).unwrap();
    data.push(b'\n');

    writer.write_all(&data).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn recv_json<R, T>(reader: &mut R) -> std::io::Result<T>
where
    R: AsyncBufRead + Unpin,
    T: DeserializeOwned,
{
    let mut line = String::new();

    reader.read_line(&mut line).await?;

    let value = serde_json::from_str(&line).unwrap();

    Ok(value)
}
