use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::Notify;

// Waker 是Rust异步编程的基石，因此绝大多数时候，我们并不需要直接去使用它。
// 例如，在 Delay 的例子中，可以使用 tokio::sync::Notify 去实现。
// 该 Notify 提供了一个基础的任务通知机制，它会处理这些 waker 的细节，包括确保两次 waker 的匹配
async fn delay(dur: Duration) {
    let when = Instant::now() + dur;
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();

    thread::spawn(move || {
        let now = Instant::now();
        if now < when {
            thread::sleep(when - now);
        }
        notify2.notify_one();
    });
    notify.notified().await;
}

#[tokio::main]
async fn main() {
    let dur = Duration::from_millis(10);
    delay(dur).await;
    println!("done")
}
