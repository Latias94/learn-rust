use std::thread::sleep;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::mpsc;

/// 发送消息
/// 在同步代码中使用异步的另一个方法就是生成一个运行时，然后使用消息传递的方式跟它进行交互。
/// 这个方法虽然更啰嗦一些，但是相对于之前的两种方法更加灵活。因为它可以在很多方面进行配置。
/// 例如，可以使用信号量 [Semaphore](https://docs.rs/tokio/1.16.1/tokio/sync/struct.Semaphore.html) 来限制当前正在进行的任务数，或者你还可以使用一个消息通道将消息反向发送回任务生成器 spawner。
/// 这种方式，其实就是 [actor](https://ryhl.io/blog/actors-with-tokio/) 的一种。
pub struct Task {
    name: String,
    // 一些信息用于描述该任务
}

async fn handle_task(task: Task) {
    println!("Got task: {}", task.name);
}

#[derive(Clone)]
pub struct TaskSpawner {
    spawn: mpsc::Sender<Task>,
}

#[allow(clippy::new_without_default)]
impl TaskSpawner {
    pub fn new() -> TaskSpawner {
        // 创建一个消息通道用于通信
        let (send, mut recv) = mpsc::channel(16);

        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        std::thread::spawn(move || {
            rt.block_on(async move {
                while let Some(task) = recv.recv().await {
                    tokio::spawn(handle_task(task));
                }

                // 一旦所有的发送端超出作用域被 drop 后，`.recv()` 方法会返回 None，同时 while 循环会退出，然后线程结束
            });
        });

        TaskSpawner { spawn: send }
    }

    pub fn spawn_task(&self, task: Task) {
        match self.spawn.blocking_send(task) {
            Ok(()) => {}
            Err(_) => panic!("The shared runtime has shut down."),
        }
    }
}

fn main() {
    let spawner = TaskSpawner::new();
    spawner.spawn_task(Task {
        name: "task1".into(),
    });
    spawner.spawn_task(Task {
        name: "task2".into(),
    });
    spawner.spawn_task(Task {
        name: "task3".into(),
    });
    sleep(Duration::new(1, 0));
}
