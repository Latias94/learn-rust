use tokio::sync::oneshot;

/// 在结果处理中进行两次可变借用时，不会报错
/// `select!` 会保证只有一个分支的结果处理会被运行，然后在运行结束后，另一个分支会被直接丢弃。
#[tokio::main]
async fn main() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();

    let mut out = String::new();

    tokio::spawn(async move {
    });

    tokio::select! {
        _ = rx1 => {
            out.push_str("rx1 completed");
        }
        _ = rx2 => {
            out.push_str("rx2 completed");
        }
    }

    println!("{}", out);
}