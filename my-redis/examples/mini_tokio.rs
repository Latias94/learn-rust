use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::process::Output;
use std::sync::{Arc, Mutex};
use std::task::Context;
use std::time::{Duration, Instant};
use crossbeam::channel;
use futures::task;
use crate::task::ArcWake;

// 我们的 mini-tokio 只应该在 Future 准备好可以进一步运行后，才去 poll 它，
// 例如该 Future 之前阻塞等待的资源已经准备好并可以被使用了，就可以对其进行 poll。
// 再比如，如果一个 Future 任务在阻塞等待从 TCP socket 中读取数据，那我们只想在 socket 中有数据可以读取后才去 poll 它
// mini-tokio 只应该当任务的延迟时间到了后，才去 poll 它。
// 为了实现这个功能，我们需要 通知 -> 运行 机制：当任务可以进一步被推进运行时，它会主动通知执行器，然后执行器再来 poll。
// 完整代码见 https://github.com/tokio-rs/website/blob/master/tutorial-code/mini-tokio/src/main.rs
fn main() {
    let mut mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async {
        let when = Instant::now() + Duration::from_millis(10);
        let future = Delay { when };

        let out = future.await;
        assert_eq!(out, "done");
    });

    mini_tokio.run();
}

struct MiniTokio {
    scheduled: channel::Receiver<Arc<Task>>,
    sender: channel::Sender<Arc<Task>>,
}

struct Task {
    // `Mutex` 是为了让 `Task` 实现 `Sync` 特征，它能保证同一时间只有一个线程可以访问 `Future`。
    // 事实上 `Mutex` 并没有在 Tokio 中被使用，这里我们只是为了简化： Tokio 的真实代码实在太长了 :D
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    executor: channel::Sender<Arc<Task>>,
}

impl Task {
    fn schedule(self: &Arc<Self>) {
        self.executor.send(self.clone());
    }

    // 使用给定的 future 来生成新的任务
    //
    // 新的任务会被推到 `sender` 中，接着该消息通道的接收端就可以获取该任务，然后执行
    fn spawn<F>(future: F, sender: &channel::Sender<Arc<Task>>)
        where F: Future<Output=()> + Send + 'static
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule()
    }
}

impl MiniTokio {
    fn new() -> MiniTokio {
        let (sender, scheduled) = channel::unbounded();
        MiniTokio { scheduled, sender }
    }

    /// 在下面函数中，通过参数传入的 future 被 `Task` 包裹起来，然后会被推入到调度队列中，当 `run` 被调用时，该 future 将被执行
    fn spawn<F>(& self, future: F)
        where
            F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    fn run(&mut self) {
        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }
}