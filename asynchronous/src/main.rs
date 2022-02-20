// see examples
mod s04_timer_future;
mod s05_executor;

pub use s04_timer_future::*;
pub use s05_executor::*;
use std::time::Duration;

fn main() {
    let (executor, spawner) = new_executor_and_spawner();
    spawner.spawn(async {
        // 创建定时器Future，并等待它完成
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done!");
    });
    // drop掉任务，这样执行器就知道任务已经完成，不会再有新的任务进来
    drop(spawner);
    // 运行执行器直到任务队列为空
    // 任务运行后，会先打印`howdy!`, 暂停2秒，接着打印 `done!`
    executor.run();
}
