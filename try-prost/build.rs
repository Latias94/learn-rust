use prost_build::Config;
use std::io::Result;

fn main() -> Result<()> {
    // OUT_DIR
    // prost_build::compile_protos(&["person.proto"], &["."])?;

    // 改变了才重新 run
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=person.proto");
    // 指定 OUT_DIR 生成在项目中
    Config::new()
        .out_dir("src/pb")
        // .bytes(&["."])
        .btree_map(&["scores"])
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .field_attribute("data", "#[serde(skip_serializing_if = \"Vec::is_empty\")]")
        .compile_protos(&["person.proto"], &["."])?;

    // https://docs.rs/prost-build/latest/prost_build/struct.Config.html
    // bytes() 需要 bytes crate. 能将生成代码的 pub data: ::prost::alloc::vec::Vec<u8>, 变成用
    // pub data: ::prost::bytes::Bytes,

    // btree_map 可以将 pub scores: ::std::collections::HashMap<::prost::alloc::string::String, i32>,
    // 变成 pub scores: ::prost::alloc::collections::BTreeMap<::prost::alloc::string::String, i32>,

    // type_attribute 可以为生成的结构添加 trait

    // field_attribute 为 field 做额外的自定义，例如上面代码可以为 data 属性跳过序列化
    Ok(())
}
