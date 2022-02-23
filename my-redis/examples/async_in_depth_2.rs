use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    // 这里的实现是有问题的，详情看 https://tokio.rs/tokio/tutorial/async#a-few-loose-ends
    // mini_tokio.rs 中有对应正确的实现
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            // 时间到了，Future 可以结束
            println!("Hello world");
            // Future 执行结束并返回 "done" 字符串
            Poll::Ready("done")
        } else {
            let waker = cx.waker().clone();
            let when = self.when;

            // 生成一个计时器线程
            // 计时器用来模拟一个阻塞等待的资源，一旦计时结束(该资源已经准备好)，
            // 资源会通过 waker.wake() 调用通知执行器我们的任务再次被调度执行了。
            thread::spawn(move || {
                let now = Instant::now();
                if now < when {
                    thread::sleep(when - now)
                }
                waker.wake();
            });
            Poll::Pending
        }
    }
}


#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_millis(10);
    let future = Delay { when };

    // 运行并等待 Future 的完成
    let out = future.await;

    // 判断 Future 返回的字符串是否是 "done"
    assert_eq!(out, "done");
}


// Delay Future 有问题的例子：
//
// use futures::future::poll_fn;
// use std::future::Future;
// use std::pin::Pin;
//
// #[tokio::main]
// async fn main() {
//     let when = Instant::now() + Duration::from_millis(10);
//     let mut delay = Some(Delay { when });
//
//     poll_fn(move |cx| {
//         let mut delay = delay.take().unwrap();
//         let res = Pin::new(&mut delay).poll(cx);
//         assert!(res.is_pending());
//         tokio::spawn(async move {
//             delay.await;
//         });
//
//         Poll::Ready(())
//     }).await;
// }