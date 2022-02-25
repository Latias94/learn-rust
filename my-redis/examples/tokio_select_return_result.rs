use std::io;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::oneshot;

/// ? 如何工作取决于它是在分支中的 async 表达式使用还是在结果处理的代码中使用:
/// * 在分支中 async 表达式使用会将该表达式的结果变成一个 Result
/// * 在结果处理中使用，会将错误直接传播到 select! 之外
#[tokio::main]
async fn main() -> io::Result<()> {
    let (tx, rx) = oneshot::channel::<String>();
    let listener = TcpListener::bind("localhost:3465").await?;

    tokio::select! {
        res = async {
            loop {
                let (socket, _) = listener.accept().await?;
                tokio::spawn(async move { process(socket) });
            }

            Ok::<_, io::Error>(())
        } => {
            res?;
        }
        _ = rx => {
            println!("terminating accept loop");
        }
    }

    Ok(())
}

fn process(stream: TcpStream) {}
