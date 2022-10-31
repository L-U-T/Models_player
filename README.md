# Models_player

Play simple 3d animation on website, powered by Rust, WASM, Wgpu(WebGPU/WebGL)

# Run

1. install `Trunk`
    ```bash
    $ cargo install trunk
    ```
1. run
    ```bash
    $ trunk server
    ```

# 目录结构

-   项目目录结构

```bash
.
├── Cargo.lock  // 不要修改此文件, 由包管理器自动维护
├── Cargo.toml  // 项目信息和依赖
├── README.md
├── index.html  // 用于打包的 html 文件
├── src         // 源码文件
│   ├── error.rs            // 站点级错误
│   ├── main.rs
│   ├── main_player         // 3d相关内容
│   │   ├── error.rs
│   │   ├── resources
│   │   └── wgpu_state.rs   // wgpu状态管理
│   ├── main_player.rs
│   └── requests            // 网络请求
│       ├── binary.rs
│       ├── image.rs
│       ├── mod.rs
│       └── text.rs
└── static      // 静态资源目录
    ├── image
    ├── obj
    ├── mtl
    └── shader
```