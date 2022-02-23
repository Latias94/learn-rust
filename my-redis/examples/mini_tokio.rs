use crossbeam::channel;
use futures::task;
use futures::task::ArcWake;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};

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
        let future = Delay::new(when);
        future.await;
        println!("done")
    });

    mini_tokio.run();

    // Output:
    // MiniTokio::new()
    // MiniTokio::spawn, call Task::spawn
    // Task::spawn, send to sender
    // MiniTokio::run, receive from scheduled, poll task
    // Task::poll, poll future
    // Delay::poll, waker is None, set new waker
    // Delay::poll, return Poll::Pending
    // Delay::poll, 计时结束，唤醒 waker
    // Task::schedule, send to sender
    // MiniTokio::run, receive from scheduled, poll task
    // Task::poll, poll future
    // Delay::poll, return Poll::Ready(())
    // done
}

struct Delay {
    when: Instant,
    // 用于说明是否已经生成一个线程
    // Some 代表已经生成， None 代表还没有
    waker: Option<Arc<Mutex<Waker>>>,
}

impl Delay {
    fn new(when: Instant) -> Self {
        Self {
            when,
            waker: None
        }
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
        println!("Task::schedule, send to sender");
        let _ = self.executor.send(self.clone());
    }

    // 使用给定的 future 来生成新的任务
    //
    // 新的任务会被推到 `sender` 中，接着该消息通道的接收端就可以获取该任务，然后执行
    fn spawn<F>(future: F, sender: &channel::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: sender.clone(),
        });
        println!("Task::spawn, send to sender");
        let _ = sender.send(task);
    }

    // 注意不是 future 的 poll，是自己实现的 poll
    fn poll(self: Arc<Self>) {
        // 基于 Task 实例创建一个 waker, 它使用了之前的 `ArcWake`
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        // 没有其他线程在竞争锁时，我们将获取到目标 future
        let mut future = self.future.try_lock().unwrap();
        println!("Task::poll, poll future");
        // 对 future 进行 poll
        let _ = future.as_mut().poll(&mut cx);
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule()
    }
}

impl MiniTokio {
    fn new() -> MiniTokio {
        println!("MiniTokio::new()");
        let (sender, scheduled) = channel::unbounded();
        MiniTokio { scheduled, sender }
    }

    /// 在下面函数中，通过参数传入的 future 被 `Task` 包裹起来，然后会被推入到调度队列中，当 `run` 被调用时，该 future 将被执行
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        println!("MiniTokio::spawn, call Task::spawn");
        Task::spawn(future, &self.sender);
    }

    fn run(&mut self) {
        while let Ok(task) = self.scheduled.recv() {
            println!("MiniTokio::run, receive from scheduled, poll task");
            task.poll();
        }
    }
}
