use tokio::sync::oneshot;
async fn some_operation() -> String {
    // 在这里执行一些操作...
    "some operation".into()
}
#[tokio::main]
async fn main() {
    // 对于 Tokio 的 oneshot 的接收端来说，它在被释放时会发送一个关闭通知到发送端，因此发送端可以通过释放任务的方式来终止正在执行的任务。
    let (mut tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();

    tokio::spawn(async {
        // 等待 `some_operation` 的完成
        // 或者处理 `oneshot` 的关闭通知
        tokio::select! {
            val = some_operation() => {
                println!("tx1.send({})", val);
                let _ = tx1.send(val);
            }
            // 重点就在于 tx1.closed 所在的分支，一旦发送端被关闭，那该分支就会被执行，然后 select 会退出，
            // 并清理掉还没执行的第一个分支 val = some_operation() ，这其中 some_operation 返回的 Future 也会被清理，
            // 根据之前的内容，Future 被清理那相应的任务会立即取消，因此 some_operation 会被取消，不再执行。
            _ = tx1.closed() => {
                // 收到了发送端发来的关闭信号
                // `select` 即将结束，此时，正在进行的 `some_operation()` 任务会被取消，任务自动完成，
                // tx1 被释放
                println!("tx1.closed()");
            }
        }
    });

    tokio::spawn(async {
        let _ = tx2.send("two");
    });

    // 这里用到了两个 oneshot 消息通道，虽然两个操作的创建在代码上有先后顺序，但在实际执行时却不这样。
    // 因此，select 在从两个通道阻塞等待接收消息时，rx1 和 rx2 都有可能被先打印出来。
    // 一旦其中任何一个 select 分支完成，就会 dropped 掉其他没执行的分支
    tokio::select! {
        val = rx1 => {
            println!("rx1 completed first with {:?}", val);
        }
        val = rx2 => {
            println!("rx2 completed first with {:?}", val);
        }
    }
}
