use tokio::runtime::Builder;
use tokio::time::{sleep, Duration};

/// 在此例中，我们生成了10个后台任务在运行时中运行，然后等待它们的完成。作为一个例子，想象一下在图形渲染应用( GUI )中，
/// 有时候需要通过网络访问远程服务来获取一些数据，那上面的这种模式就非常适合，因为这些网络访问比较耗时，
/// 而且不会影响图形的主体渲染，因此可以在主线程中渲染图形，然后使用其它线程来运行 Tokio 的运行时，
/// 并通过该运行时使用异步的方式完成网络访问，最后将这些网络访问的结果发送到 GUI 进行数据渲染，例如一个进度条。
///
/// 还有一点很重要，在本例子中只能使用 multi_thread 运行时。如果我们使用了 current_thread，
/// 你会发现主线程的耗时任务会在后台任务开始之前就完成了。因为在 current_thread 模式下，生成的任务只会在 block_on 期间才执行。
///
/// 在 multi_thread 模式下，我们并不需要通过 block_on 来触发任务的运行，这里仅仅是用来阻塞并等待最终的结果。
/// 而除了通过 block_on 等待结果外，你还可以：
/// * 使用消息传递的方式，例如 tokio::sync::mpsc，让异步任务将结果发送到主线程，然后主线程通过 .recv方法等待这些结果
/// * 通过共享变量的方式，例如 Mutex，这种方式非常适合实现 GUI 的进度条: GUI 在每个渲染帧读取该变量即可。
fn main() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let mut handles = Vec::with_capacity(10);
    for i in 0..10 {
        handles.push(runtime.spawn(my_bg_task(i)));
    }

    // 在后台任务运行的同时做一些耗费时间的事情
    std::thread::sleep(Duration::from_millis(750));
    println!("Finished time-consuming task.");

    // 等待这些后台任务的完成
    for handle in handles {
        // `spawn` 方法返回一个 `JoinHandle`，它是一个 `Future`，因此可以通过  `block_on` 来等待它完成
        runtime.block_on(handle).unwrap();
    }
}

async fn my_bg_task(i: u64) {
    let millis = 1000 - 50 * i;
    println!("Task {} sleeping for {} ms.", i, millis);

    sleep(Duration::from_millis(millis)).await;

    println!("Task {} stopping.", i);
}

// Task 0 sleeping for 1000 ms.
// Task 1 sleeping for 950 ms.
// Task 2 sleeping for 900 ms.
// Task 3 sleeping for 850 ms.
// Task 4 sleeping for 800 ms.
// Task 5 sleeping for 750 ms.
// Task 6 sleeping for 700 ms.
// Task 7 sleeping for 650 ms.
// Task 8 sleeping for 600 ms.
// Task 9 sleeping for 550 ms.
// Task 9 stopping.
// Task 8 stopping.
// Task 7 stopping.
// Task 6 stopping.
// Finished time-consuming task.
// Task 5 stopping.
// Task 4 stopping.
// Task 3 stopping.
// Task 2 stopping.
// Task 1 stopping.
// Task 0 stopping.