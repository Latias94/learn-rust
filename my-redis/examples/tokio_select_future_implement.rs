use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;

/// 为了更好的理解 select 的工作原理，我们来看看如果使用 Future 该如何实现。当然，这里是一个简化版本，
/// 在实际中，select! 会包含一些额外的功能，例如一开始会随机选择一个分支进行 poll。
///
/// MySelect 包含了两个分支中的 Future，当它被 poll 时，第一个分支会先执行。如果执行完成，那取出的值会被使用，然后 MySelect 也随之结束。
/// 而另一个分支对应的 Future 会被释放掉，对应的操作也会被取消。
struct MySelect {
    rx1: oneshot::Receiver<&'static str>,
    rx2: oneshot::Receiver<&'static str>,
}

impl Future for MySelect {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if let Poll::Ready(val) = Pin::new(&mut self.rx1).poll(cx) {
            println!("rx1 completed first with {:?}", val);
            return Poll::Ready(());
        }

        if let Poll::Ready(val) = Pin::new(&mut self.rx2).poll(cx) {
            println!("rx2 completed first with {:?}", val);
            return Poll::Ready(());
        }

        Poll::Pending
    }
}
#[tokio::main]
async fn main() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();

    // 使用 tx1 和 tx2

    MySelect { rx1, rx2 }.await;
}
