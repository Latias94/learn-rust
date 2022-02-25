use std::io;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    race(
        b"hi",
        "127.0.0.1:8080".parse().unwrap(),
        "127.0.0.1:8081".parse().unwrap(),
    )
    .await
}

/// 在结果处理中进行两次可变借用时，不会报错
/// `select!` 会保证只有一个分支的结果处理会被运行，然后在运行结束后，另一个分支会被直接丢弃。
async fn race(data: &[u8], addr1: SocketAddr, addr2: SocketAddr) -> io::Result<()> {
    tokio::select! {
        Ok(_) = async {
            let mut socket = TcpStream::connect(addr1).await?;
            socket.write_all(data).await?;
            Ok::<_, io::Error>(())
        } => {}
        Ok(_) = async {
            let mut socket = TcpStream::connect(addr2).await?;
            socket.write_all(data).await?;
            Ok::<_, io::Error>(())
        } => {}
        else => {}
    };

    Ok(())
}
