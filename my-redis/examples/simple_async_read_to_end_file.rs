use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut f = File::open("Cargo.toml").await?;
    let mut buffer = Vec::new();
    // AsyncReadExt::read_to_end 方法会从字节流中读取所有的字节，直到遇到 EOF
    f.read_to_end(&mut buffer).await?;
    // println!("The bytes: {:?}", &buffer[..]);
    println!("The String: {:?}", std::str::from_utf8(&buffer).unwrap());
    Ok(())
}
