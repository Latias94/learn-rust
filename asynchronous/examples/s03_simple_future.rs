// 这个例子展示了在不需要内存对象分配以及深层嵌套回调的情况下，该如何使用 Future 特征去表达异步控制流。

// 若在当前 poll 中， Future 可以被完成，则会返回 Poll::Ready(result) ，反之则返回 Poll::Pending，
// 并且安排一个 wake 函数：当未来 Future 准备好进一步执行时， 该函数会被调用，然后管理该 Future 的执行器
// (例如上一章节中的block_on函数)会再次调用 poll 方法，此时 Future 就可以继续执行了。

// 果没有 wake 方法，那执行器无法知道某个Future是否可以继续被执行，除非执行器定期的轮询每一个 Future ，
// 确认它是否能被执行，但这种作法效率较低。而有了 wake，Future 就可以主动通知执行器，然后执行器就可以精确的执行该 Future。
// 这种“事件通知 -> 执行”的方式要远比定期对所有 Future 进行一次全遍历来的高效。
trait SimpleFuture {
    type Output;
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),
    Pending,
}
struct Socket;
impl Socket {
    fn has_data_to_read(&self) -> bool {
        // check if the socket is currently readable
        true
    }
    fn read_buf(&self) -> Vec<u8> {
        // Read data in from the socket
        vec![]
    }
    fn set_readable_callback(&self, _wake: fn()) {
        // register `_wake` with something that will call it
        // once the socket becomes readable, such as an
        // `epoll`-based event loop.
    }
}
pub struct SocketRead<'a> {
    socket: &'a Socket,
}

impl SimpleFuture for SocketRead<'_> {
    type Output = Vec<u8>;

    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if self.socket.has_data_to_read() {
            // socket有数据，写入buffer中并返回
            Poll::Ready(self.socket.read_buf())
        } else {
            // socket中还没数据
            //
            // 注册一个`wake`函数，当数据可用时，该函数会被调用，
            // 然后当前Future的执行器会再次调用`poll`方法，此时就可以读取到数据
            self.socket.set_readable_callback(wake);
            Poll::Pending
        }
    }
}

/// 一个SimpleFuture，它会并发地运行两个Future直到它们完成
/// 之所以可以并发，是因为两个Future的轮询可以交替进行，一个阻塞，另一个就可以立刻执行，反之亦然
pub struct Join<FutureA, FutureB> {
    // 结构体的每个字段都包含一个Future，可以运行直到完成.
    // 如果Future完成后，字段会被设置为 `None`. 这样Future完成后，就不会再被轮询
    a: Option<FutureA>,
    b: Option<FutureB>,
}

/// 展示了如何同时运行多个 Future， 且在此过程中没有任何内存分配
impl<FutureA, FutureB> SimpleFuture for Join<FutureA, FutureB>
where
    FutureA: SimpleFuture<Output = ()>,
    FutureB: SimpleFuture<Output = ()>,
{
    type Output = ();

    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        // 尝试去完成一个 Future `a`
        if let Some(a) = &mut self.a {
            if let Poll::Ready(()) = a.poll(wake) {
                self.a.take();
            }
        }
        // 尝试去完成一个 Future `b`
        if let Some(b) = &mut self.b {
            if let Poll::Ready(()) = b.poll(wake) {
                self.b.take();
            }
        }

        if self.a.is_none() && self.b.is_none() {
            // 两个 Future都已完成 - 我们可以成功地返回了
            Poll::Ready(())
        } else {
            // 至少还有一个 Future 没有完成任务，因此返回 `Poll::Pending`.
            // 当该 Future 再次准备好时，通过调用`wake()`函数来继续执行
            Poll::Pending
        }
    }
}

/// 一个SimpleFuture, 它使用顺序的方式，一个接一个地运行两个Future
//
// 注意: 由于本例子用于演示，因此功能简单，`AndThenFut` 会假设两个 Future 在创建时就可用了.
// 而真实的`Andthen`允许根据第一个`Future`的输出来创建第二个`Future`，因此复杂的多。
pub struct AndThenFut<FutureA, FutureB> {
    first: Option<FutureA>,
    second: FutureB,
}

impl<FutureA, FutureB> SimpleFuture for AndThenFut<FutureA, FutureB>
where
    FutureA: SimpleFuture<Output = ()>,
    FutureB: SimpleFuture<Output = ()>,
{
    type Output = ();
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if let Some(first) = &mut self.first {
            match first.poll(wake) {
                // 我们已经完成了第一个 Future， 可以将它移除， 然后准备开始运行第二个
                Poll::Ready(()) => self.first.take(),
                // 第一个 Future 还不能完成
                Poll::Pending => return Poll::Pending,
            };
        }

        // 运行到这里，说明第一个Future已经完成，尝试去完成第二个
        self.second.poll(wake)
    }
}
fn main() {}

mod real_future {
    use std::{
        future::Future as RealFuture,
        pin::Pin,
        task::{Context, Poll},
    };

    // ANCHOR: real_future
    trait Future {
        type Output;
        fn poll(
            // 首先值得注意的地方是，`self`的类型从`&mut self`变成了`Pin<&mut Self>`:
            // 创建一个无法被移动的 Future ，因为无法被移动，因此它将具有固定的内存地址，意味着我们可以存储它的指针
            // (如果内存地址可能会变动，那存储指针地址将毫无意义！)，也意味着可以实现一个自引用数据结构：struct MyFut { a: i32, ptr_to_a: *const i32 }
            self: Pin<&mut Self>,
            // 其次将`wake: fn()` 修改为 `cx: &mut Context<'_>`:
            // 意味着 wake 函数可以携带数据了
            // 如果不能携带数据，当一个 Future 调用 wake 后，执行器该如何知道是哪个 Future 调用了 wake ,然后进一步去 poll 对应的 Future ？
            // 总之，在正式场景要进行 wake ，就必须携带上数据。 而 Context 类型通过提供一个 Waker 类型的值，就可以用来唤醒特定的的任务。
            cx: &mut Context<'_>,
        ) -> Poll<Self::Output>;
    }
// ANCHOR_END: real_future

    // ensure that `Future` matches `RealFuture`:
    impl<O> Future for dyn RealFuture<Output = O> {
        type Output = O;
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            RealFuture::poll(self, cx)
        }
    }
}