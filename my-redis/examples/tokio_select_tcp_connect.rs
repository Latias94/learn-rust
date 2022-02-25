use tokio::net::TcpStream;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel();
    tokio::spawn(async {
        tx.send("done").unwrap();
    });

    // select! 最多可以支持 64 个分支，每个分支形式如下：
    // <模式> = <async 表达式> => <结果处理>,
    // 当 select 宏开始执行后，所有的分支会开始并发的执行。当任何一个表达式完成时，会将结果跟模式进行匹配。若匹配成功，则剩下的表达式会被释放。
    tokio::select! {
        socket = TcpStream::connect("localhost:3465") => {
            println!("Socket connected {:?}", socket);
        }
        msg = rx => {
            println!("received message first {:?}", msg);
        }
    }
}
