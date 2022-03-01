use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            // 此处的缓冲区是一个 Vec 动态数组，它的数据是存储在堆上，而不是栈上(若改成 let mut buf = [0; 1024];，则存储在栈上)。
            // 一个数据如果想在 .await 调用过程中存在，那它必须存储在当前任务内。
            // 在我们的代码中，buf 会在 .await 调用过程中被使用，因此它必须要存储在任务内。
            // 当任务因为调度在线程间移动时，存储在栈上的数据需要进行保存和恢复，过大的栈上变量会带来不小的数据拷贝开销
            // 因此，存储大量数据的变量最好放到堆上
            let mut buf = vec![0; 1024];
            loop {
                match socket.read(&mut buf).await {
                    // 返回值 `Ok(0)` 说明对端已经关闭{}
                    Ok(0) => return,
                    Ok(n) => {
                        // Copy the data back to socket
                        // 将数据拷贝回 socket 中
                        if socket.write_all(&buf[..n]).await.is_err() {
                            // 非预期错误，由于我们这里无需再做什么，因此直接停止处理
                            return;
                        }
                    }
                    Err(_) => {
                        // 非预期错误，由于我们无需再做什么，因此直接停止处理
                        return;
                    }
                }
            }
        });
    }
}
