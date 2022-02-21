use async_std::io::{Read, Write};
use async_std::net::TcpListener;
use async_std::task::spawn;
use futures::io::Error;
use futures::stream::StreamExt;
use futures::task::Context;
use futures::{AsyncReadExt, AsyncWriteExt};

use std::task::Poll;

use async_std::task;
use std::cmp::min;
use std::pin::Pin;
use std::time::Duration;

const HTML_404: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf - 8">
    <title>Hello!</title>
</head>
<body>
<h1>Oops!</h1>
<p>Sorry, I don't know what you're asking for.</p>
</body>
</html>"#;

const HTML_HELLO: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>Hello!</title>
</head>
<body>
<h1>Hello!</h1>
<p>Hi from Rust</p>
</body>
</html>"#;

#[async_std::main]
async fn main() {
    // 监听本地端口 7878 ，等待 TCP 连接的建立
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

    // std::net::TcpListener 的 listener.incoming() 是阻塞的迭代器
    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();
    //
    //     handle_connection(stream).await;
    // }

    // 异步版本的 TcpListener 为 listener.incoming() 实现了 Stream 特征
    // 1. listener.incoming() 不再阻塞
    // 2. 使用 for_each_concurrent 并发地处理从 Stream 获取的元素
    // 至此，我们实现了同时使用并行(多线程)和并发( async )来同时处理多个请求！
    listener
        .incoming()
        .for_each_concurrent(/* limit */ None, |tcpstream| async move {
            let tcpstream = tcpstream.unwrap();
            spawn(handle_connection(tcpstream));
        })
        .await;
}

async fn handle_connection(mut stream: impl Read + Write + Unpin) {
    // 从连接中顺序读取 1024 字节数据
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await.unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    // 处理HTTP协议头，若不符合则返回404和对应的`html`文件
    let (status_line, contents) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", HTML_HELLO)
    } else if buffer.starts_with(sleep) {
        task::sleep(Duration::from_secs(5)).await;
        ("HTTP/1.1 200 OK\r\n\r\n", HTML_HELLO)
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", HTML_404)
    };

    // 将回复内容写入连接缓存中
    let response = format!("{status_line}{contents}");
    stream.write(response.as_bytes()).await.unwrap();
    // 使用flush将缓存中的内容发送到客户端
    stream.flush().await.unwrap();
}

struct MockTcpStream {
    read_data: Vec<u8>,
    write_data: Vec<u8>,
}

impl Read for MockTcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        let size: usize = min(self.read_data.len(), buf.len());
        buf[..size].copy_from_slice(&self.read_data[..size]);
        Poll::Ready(Ok(size))
    }
}
impl Write for MockTcpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        self.write_data = Vec::from(buf);

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }
}
impl Unpin for MockTcpStream {}

#[async_std::test]
async fn test_handle_connection() {
    let input_bytes = b"GET / HTTP/1.1\r\n";
    let mut contents = vec![0u8; 1024];
    contents[..input_bytes.len()].clone_from_slice(input_bytes);
    let mut stream = MockTcpStream {
        read_data: contents,
        write_data: Vec::new(),
    };

    handle_connection(&mut stream).await;
    let mut buf = [0u8; 1024];
    stream.read(&mut buf).await.unwrap();

    let expected_contents = HTML_HELLO;
    let expected_response = format!("HTTP/1.1 200 OK\r\n\r\n{}", expected_contents);
    assert!(stream.write_data.starts_with(expected_response.as_bytes()));
}
