async fn computation1() -> String {
    "computation1".into()
}

async fn computation2() -> String {
    "computation2".into()
}

/// select! 还能返回一个值
/// 需要注意的是，此时 select! 的所有分支必须返回一样的类型，否则编译器会报错
#[tokio::main]
async fn main() {
    let out = tokio::select! {
        res1 = computation1() => res1,
        res2 = computation2() => res2,
    };

    println!("Got = {}", out);
}
