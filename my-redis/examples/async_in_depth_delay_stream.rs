use futures::task::ArcWake;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};
use tokio_stream::{Stream, StreamExt};

/// 还记得在深入async 中构建的 Delay Future 吗？现在让我们来更进一步，
/// 将它转换成一个 stream，每 10 毫秒生成一个值，总共生成 3 次:
struct Delay {
    when: Instant,
    // 用于说明是否已经生成一个线程
    // Some 代表已经生成， None 代表还没有
    waker: Option<Arc<Mutex<Waker>>>,
}

impl Delay {
    fn new(when: Instant) -> Self {
        Self { when, waker: None }
    }
}

// 当实现一个 Future 时，很关键的一点就是要假设每次 poll 调用都会应用到一个不同的 Waker 实例上。
// 因此 poll 函数必须要使用一个新的 waker 去更新替代之前的 waker。
//
// 我们之前的 Delay 实现中，会在每一次 poll 调用时都生成一个新的线程。这么做问题不大，但是当 poll 调用较多时会出现明显的性能问题！
// 一个解决方法就是记录你是否已经生成了一个线程，然后只有在没有生成时才去创建一个新的线程。
// 但是一旦这么做，就必须确保线程的 Waker 在后续 poll 调用中被正确更新，否则你无法唤醒最近的 Waker！
impl Future for Delay {
    type Output = ();

    // 在每次 poll 调用时，都会检查 Context 中提供的 waker 和我们之前记录的 waker 是否匹配。
    // 若匹配，就什么都不用做。
    // 若不匹配，那之前存储的就必须进行更新。
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 若这是 Future 第一次被调用，那么需要先生成一个计时器线程。
        // 若不是第一次调用(该线程已在运行)，那要确保已存储的 `Waker` 跟当前任务的 `waker` 匹配
        if let Some(waker) = &self.waker {
            let mut waker = waker.lock().unwrap();
            // 检查之前存储的 `waker` 是否跟当前任务的 `waker` 相匹配.
            // 这是必要的，原因是 `Delay Future` 的实例可能会在两次 `poll` 之间被转移到另一个任务中，然后
            // 存储的 waker 被该任务进行了更新。
            // 这种情况一旦发生，`Context` 包含的 `waker` 将不同于存储的 `waker`。
            // 因此我们必须对存储的 `waker` 进行更新
            if !waker.will_wake(cx.waker()) {
                println!("Delay::poll, waker is different from origin, set new waker to self");
                *waker = cx.waker().clone();
            }
        } else {
            println!("Delay::poll, waker is None, set new waker");
            let when = self.when;
            let waker = Arc::new(Mutex::new(cx.waker().clone()));
            self.waker = Some(waker.clone());

            // 第一次调用 `poll`，生成计时器线程
            // 生成一个计时器线程
            // 计时器用来模拟一个阻塞等待的资源，一旦计时结束(该资源已经准备好)，
            // 资源会通过 waker.wake() 调用通知执行器我们的任务再次被调度执行了。
            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }

                // 计时结束，通过调用 `waker` 来通知执行器
                let waker = waker.lock().unwrap();
                println!("Delay::poll, 计时结束，唤醒 waker");
                waker.wake_by_ref();
            });
        }

        // 一旦 waker 被存储且计时器线程已经开始，我们就需要检查 `delay` 是否已经完成
        // 若计时已完成，则当前 Future 就可以完成并返回 `Poll::Ready`
        if Instant::now() >= self.when {
            println!("Delay::poll, return Poll::Ready(())");
            Poll::Ready(())
        } else {
            println!("Delay::poll, return Poll::Pending");
            // 计时尚未结束，Future 还未完成，因此返回 `Poll::Pending`。
            // 在我们的例子中，会通过生成的计时线程来保证。如果忘记调用 waker，该任务将被永远的挂起，无法再执行。
            Poll::Pending
        }
    }
}

struct Interval {
    rem: usize,
    delay: Delay,
}

// Stream::poll_next() 函数跟 Future::poll 很相似，区别就是前者为了从 stream 收到多个值需要重复的进行调用。
// 就像在 深入async 章节提到的那样，当一个 stream 没有做好返回一个值的准备时，它将返回一个 Poll::Pending ，
// 同时将任务的 waker 进行注册。一旦 stream 准备好后， waker 将被调用。
impl Stream for Interval {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.rem == 0 {
            // 去除计时器实现
            return Poll::Ready(None);
        }
        match Pin::new(&mut self.delay).poll(cx) {
            Poll::Ready(_) => {
                let when = self.delay.when + Duration::from_millis(1000);
                self.delay = Delay::new(when);
                self.rem -= 1;
                Poll::Ready(Some(()))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// 手动实现 Stream 特征实际上是相当麻烦的事，不幸地是，Rust 语言的 async/await 语法目前还不能用于定义 stream，虽然相关的工作已经在进行中。
// 作为替代方案，async-stream 包提供了一个 stream! 宏，它可以将一个输入转换成 stream，使用这个包，上面的代码可以这样实现：
// use async_stream::stream;
// use std::time::{Duration, Instant};
//
// stream! {
//     let mut when = Instant::now();
//     for _ in 0..3 {
//         let delay = Delay { when };
//         delay.await;
//         yield ();
//         when += Duration::from_millis(10);
//     }
// }

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_millis(1000);
    let interval = Interval {
        rem: 3,
        delay: Delay::new(when),
    };
    tokio::pin!(interval);

    while let Some(_) = interval.next().await {
        println!("interval.rem = {}", interval.rem);
    }
    println!("DONE")
}
