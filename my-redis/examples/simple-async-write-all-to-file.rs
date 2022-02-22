use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut file = File::create("foo.txt").await?;
    // AsyncWriteExt::write_all 将缓冲区的内容全部写入到写入器中
    file.write_all(b"some bytes").await?;
    Ok(())
}
