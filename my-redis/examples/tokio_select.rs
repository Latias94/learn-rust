use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();
    tokio::spawn(async {
        let _ = tx1.send("one");
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