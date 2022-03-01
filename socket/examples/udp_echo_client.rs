use socket2::{Domain, Protocol, Socket, Type};
use std::{io, thread};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

fn main() -> io::Result<()> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    let client_src: SocketAddr = "127.0.0.1:50001".parse().unwrap();
    let server_src: SocketAddr = "127.0.0.1:50000".parse().unwrap();
    // 通知操作系统 socket 将使用一个特定地址和传输层端口的过程称为绑定（binding）
    // 不绑定的话，socket 库会找一个未被使用的端口并绑定。或者用 "127.0.0.1:0" 做地址，让库来找端口。
    socket.bind(&client_src.into())?;
    let udp_socket: UdpSocket = socket.into();
    let send_buf = "hello".as_bytes();
    if udp_socket.send_to(send_buf, &server_src).is_err() {
        // 非预期错误，由于我们这里无需再做什么，因此直接停止处理
        eprintln!("error")
    }
    let mut buf = vec![0; 1024];
    loop {
        match udp_socket.recv_from(&mut buf) {
            // 另一端发送了 FIN 数据包，承诺没有更多需要发送的数据
            Ok((0, _)) => {
                println!("finish")
            }
            Ok((n, src)) => {
                // 将数据拷贝回 socket 中
                let s = match std::str::from_utf8(&buf[..n]) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                };
                println!("client accept: {}", s);
                thread::sleep(Duration::from_secs(1));
                if udp_socket.send_to(&buf[..n], &src).is_err() {
                    // 非预期错误，由于我们这里无需再做什么，因此直接停止处理
                    eprintln!("error1")
                }
            }
            Err(_) => {
                // 非预期错误，由于我们无需再做什么，因此直接停止处理
                // eprintln!("error2 {}", e)
            }
        }
    }
}
