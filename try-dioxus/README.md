# Dioxus

参考 [Rust crates 大巡礼：用 dioxus 开发 WASM 前端](https://www.bilibili.com/video/BV1iL4y1g7kR) 。

运行前记得添加 WASM 的 target。

```shell
rustup target add wasm32-unknown-unknown
```

尝试 [Dioxus](https://github.com/DioxusLabs/dioxus) 构建 WASM 应用。

用到 [Trunk](https://trunkrs.dev/) 为 main 函数生成 WASM 的绑定。

以下命令会在 `./dist` 目录下生成构建应用。

```shell
trunk serve
```
