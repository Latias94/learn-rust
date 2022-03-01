use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::net::{SocketAddr, UdpSocket};

fn main() -> io::Result<()> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    let address: SocketAddr = "127.0.0.1:50000".parse().unwrap();
    socket.bind(&address.into())?;
    let udp_socket: UdpSocket = socket.into();
    loop {
        let mut buf = vec![0; 1024];
        loop {
            match udp_socket.recv_from(&mut buf) {
                Ok((0, _)) => {
                    println!("finish")
                }
                Ok((n, src)) => {
                    // 将数据拷贝回 socket 中
                    let s = match std::str::from_utf8(&buf[..n]) {
                        Ok(v) => v,
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };
                    println!("server accept :{} from {}", s, src);
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
}
