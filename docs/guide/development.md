# 开发构建

## 环境要求

- Rust 1.85 或更高版本（edition 2024）
- Windows SDK（用于 Win32 API 与 `windows` crate）
- Cargo

## 克隆仓库

```bash
git clone https://github.com/eeymoo/peregrine.git
cd peregrine
```

## 构建

```bash
# 调试构建
cargo build

# 发布构建（体积小、性能高）
cargo build --release

# 运行 GUI 主程序
cargo run -p peregrine
```

## 测试

```bash
# 运行全部测试
cargo test

# 只运行配置库测试
cargo test -p peregrine_config
```

## 代码检查

```bash
cargo fmt
cargo clippy
```

## 发布产物

`cargo build --release` 生成的可执行文件位于：

```
target/release/peregrine.exe
```

发布版本的编译选项已针对体积与性能优化：

- `opt-level = "z"`
- `lto = true`
- `codegen-units = 1`
- `strip = true`
- `panic = "abort"`
