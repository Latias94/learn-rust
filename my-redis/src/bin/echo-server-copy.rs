use tokio::io;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            // 根据使用场景，由于 io::copy() 调用时所在的任务和 split 所在的任务是同一个，因此可以使用性能最高的 TcpStream::split
            let (mut rd, mut wr) = socket.split();
            match io::copy(&mut rd, &mut wr).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        });
    }
}
