use mini_redis::client;
use tokio_stream::StreamExt;

/// 实现一个复杂一些的 mini-redis 客户端
/// 开始前先命令行启动 mini-redis-server
/// 完整代码见 https://github.com/tokio-rs/website/blob/master/tutorial-code/streams/src/main.rs

async fn publish() -> mini_redis::Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;

    // 发布一些数据
    client.publish("numbers", "1".into()).await?;
    client.publish("numbers", "two".into()).await?;
    client.publish("numbers", "3".into()).await?;
    client.publish("numbers", "four".into()).await?;
    client.publish("numbers", "five".into()).await?;
    client.publish("numbers", "6".into()).await?;
    Ok(())
}

async fn subscribe() -> mini_redis::Result<()> {
    let client = client::connect("127.0.0.1:6379").await?;
    let subscriber = client.subscribe(vec!["numbers".to_string()]).await?;
    // into_stream 会将 Subscriber 变成一个 stream
    let messages = subscriber
        .into_stream()
        .filter(|msg| matches!(msg, Ok(msg) if msg.content.len() == 1))
        // map 中的 msg 只能是 Ok(...)，因此 unwrap 非常安全。
        .map(|msg| msg.unwrap().content)
        .take(3);
    // 在 stream 上调用 next 方法要求该 stream 被固定住(pinned)，因此需要调用 tokio::pin!
    tokio::pin!(messages);
    while let Some(msg) = messages.next().await {
        println!("got = {:?}", msg);
    }

    Ok(())
}

/// 生成了一个异步任务专门用于发布消息到 min-redis 服务器端的 numbers 消息通道中。
/// 然后，在 main 中，我们订阅了 numbers 消息通道，并且打印从中接收到的消息。
#[tokio::main]
async fn main() -> mini_redis::Result<()> {
    tokio::spawn(async { publish().await });

    subscribe().await?;

    println!("DONE");
    Ok(())
}
