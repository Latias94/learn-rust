use tokio::sync::mpsc;

/// 在循环中使用 select! 最大的不同就是，当某一个分支执行完成后，select! 会继续循环等待并执行下一个分支，
/// 直到所有分支最终都完成，最终匹配到 else 分支，然后通过 break 跳出循环。
#[tokio::main]
async fn main() {
    let (tx1, mut rx1) = mpsc::channel(128);
    let (tx2, mut rx2) = mpsc::channel(128);
    let (tx3, mut rx3) = mpsc::channel(128);

    loop {
        let msg = tokio::select! {
            Some(msg) = rx1.recv() => msg,
            Some(msg) = rx2.recv() => msg,
            Some(msg) = rx3.recv() => msg,
            else => { break }
        };

        println!("Got {:?}", msg);
    }

    println!("All channels have been closed.");
}