use tokio::signal;

/// 如果你的服务是一个小说阅读网站，那大概率用不到优雅关闭的，简单粗暴的关闭服务器，然后用户再次请求时获取一个错误就是了。
/// 但如果是一个web服务或数据库服务呢？当前的连接很可能在做着重要的事情，一旦关闭会导致数据的丢失甚至错误，
/// 此时，我们就需要优雅的关闭(graceful shutdown)了。
///
/// 要让一个异步应用优雅的关闭往往需要做到3点：
/// * 找出合适的关闭时机
/// * 通知程序的每一个子部分开始关闭
/// * 在主线程等待各个部分的关闭结果
///
/// 可以参考 mini-redis 的完整实现，特别是 [src/server.rs](https://github.com/tokio-rs/mini-redis/blob/master/src/server.rs) 和 [src/shutdown.rs](https://github.com/tokio-rs/mini-redis/blob/master/src/shutdown.rs) 。
#[tokio::main]
async fn main() {
    // ... spawn application as separate task ...
    // 在一个单独的任务中处理应用逻辑

    match signal::ctrl_c().await {
        Ok(()) => {},
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        },
    }

    //  发送关闭信号给应用所在的任务，然后等待
}

// use tokio::sync::mpsc::{channel, Sender};
// use tokio::time::{sleep, Duration};
//
// #[tokio::main]
// async fn main() {
//     let (send, mut recv) = channel(1);
//
//     for i in 0..10 {
//         tokio::spawn(some_operation(i, send.clone()));
//     }
//
//     // 等待各个任务的完成
//     //
//     // 我们需要 drop 自己的发送端，因为等下的 `recv()` 调用会阻塞, 如果不 `drop` ，那发送端就无法被全部关闭
//     // `recv` 也将永远无法结束，这将陷入一个类似死锁的困境
//     drop(send);
//
//     // 当所有发送端都超出作用域被 `drop` 时 (当前的发送端并不是因为超出作用域被 `drop` 而是手动 `drop` 的)
//     // `recv` 调用会返回一个错误
//     let _ = recv.recv().await;
// }
//
// async fn some_operation(i: u64, _sender: Sender<()>) {
//     sleep(Duration::from_millis(100 * i)).await;
//     println!("Task {} shutting down.", i);
//
//     // 发送端超出作用域，然后被 `drop`
// }

// 关于忘记 drop 本身持有的发送端进而导致 bug 的问题，可以看看 https://course.rs/pitfalls/main-with-channel-blocked.html