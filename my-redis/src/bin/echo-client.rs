use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// echo server 会将用户的输入内容直接返回给用户，就像回声壁一样。
///
/// io::split 可以用于任何同时实现了 AsyncRead 和 AsyncWrite 的值，它的内部使用了 Arc 和 Mutex 来实现相应的功能。
/// 如果大家觉得这种实现有些重，可以使用 Tokio 提供的 TcpStream，它提供了两种方式进行分离:
/// * TcpStream::split 会获取字节流的引用，然后将其分离成一个读取器和写入器。但由于使用了引用的方式，
/// 它们俩必须和 split 在同一个任务中。优点就是，这种实现没有性能开销，因为无需 Arc 和 Mutex。
/// * TcpStream::into_split 还提供了一种分离实现，分离出来的结果可以在任务间移动，内部是通过 Arc 实现。
#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = TcpStream::connect("127.0.0.1:6142").await?;
    // TcpStream 实现 AsyncRead 和 AsyncWrite，我们需要将其功能分离来用。
    // io::copy(&mut socket, &mut socket).await // fails to compile
    let (mut rd, mut wr) = io::split(socket);
    tokio::spawn(async move {
        wr.write_all(b"hello\r\n").await?;
        wr.write_all(b"world\r\n").await?;

        // 有时，我们需要给予 Rust 一些类型暗示，它才能正确的推导出类型
        Ok::<_, io::Error>(())
    });
    let mut buf = vec![0; 128];

    loop {
        let n = rd.read(&mut buf).await?;

        if n == 0 {
            break;
        }

        println!("GOT {:?}", std::str::from_utf8(&buf[..n]));
    }
    Ok(())
}
