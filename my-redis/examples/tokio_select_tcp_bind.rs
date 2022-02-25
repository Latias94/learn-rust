use std::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

/// 分支中接收连接的循环会一直运行，直到遇到错误才停止，或者当 rx 中有值时，也会停止。
/// _ 表示我们并不关心这个值，这样使用唯一的目的就是为了结束第一分支中的循环。
#[tokio::main]
async fn main() -> io::Result<()> {
    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        tx.send(()).unwrap();
    });

    let mut listener = TcpListener::bind("localhost:3465").await?;

    tokio::select! {
        _ = async {
            loop {
                let (socket, _) = listener.accept().await?;
                tokio::spawn(async move { process(socket) });
            }

            // 给予 Rust 类型暗示
            Ok::<_, io::Error>(())
        } => {}
        _ = rx => {
            println!("terminating accept loop");
        }
    }

    Ok(())
}

fn process(stream: TcpStream) {}
