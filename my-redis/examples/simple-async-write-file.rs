use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut file = File::create("foo.txt").await?;
    // b"some bytes" 这种写法可以将一个 &str 字符串转变成一个字节数组：&[u8;10]
    // 然后 write 方法又会将这个 &[u8;10] 的数组类型隐式强转为数组切片: &[u8]。
    let n = file.write(b"some bytes").await?;
    println!("Wrote the first {} bytes of 'some bytes'.", n);
    Ok(())
}
