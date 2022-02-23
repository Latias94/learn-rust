use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            // 时间到了，Future 可以结束
            println!("Hello world");
            // Future 执行结束并返回 "done" 字符串
            Poll::Ready("done")
        } else {
            // 由于此时我们还没有模拟计时资源，因此这里直接调用了 wake 进行通知，这样做会导致当前的 Future 被立即再次调度执行。
            cx.waker().wake_by_ref();
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

// 生成的代码类似下面：
enum MainFuture {
    // 初始化，但永远不会被 poll
    State0,
    // 等待 `Delay` 运行，例如 `future.await` 代码行
    State1(Delay),
    // Future 执行完成
    Terminated,
}

impl Future for MainFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()>
    {
        use MainFuture::*;

        loop {
            match *self {
                State0 => {
                    let when = Instant::now() +
                        Duration::from_millis(10);
                    let future = Delay { when };
                    *self = State1(future);
                }
                State1(ref mut my_future) => {
                    // 当一个 Future 由其它 Future 组成时，调用外层 Future 的 poll 函数会同时调用一次内部 Future 的 poll 函数。
                    match Pin::new(my_future).poll(cx) {
                        Poll::Ready(out) => {
                            assert_eq!(out, "done");
                            *self = Terminated;
                            return Poll::Ready(());
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    }
                }
                Terminated => {
                    panic!("future polled after completion")
                }
            }
        }
    }
}