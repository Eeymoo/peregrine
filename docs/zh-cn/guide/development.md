# 开发构建

## 环境要求

- Rust 1.85 或更高版本（edition 2024）
- Node.js 20 或更高版本（前端构建）
- Windows SDK（用于 Win32 API 与 `windows` crate）
- Cargo

## 克隆仓库

```bash
git clone https://github.com/eeymoo/peregrine.git
cd peregrine
```

## 构建

```bash
# 安装前端依赖
npm install

# 调试构建
cargo build

# 发布构建（体积小、性能高）
cargo build --release

# 运行 Tauri 开发版本（带热更新）
npx tauri dev

# 构建 Tauri release 安装包
npx tauri build
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
cargo clippy -p peregrine_config -- -D warnings
```

## 发布产物

`npx tauri build` 生成的 release 产物位于 `src-tauri/target/release/` 目录下，MSI 安装包位于 `src-tauri/target/release/bundle/msi/`。

发布版本的编译选项已针对体积与性能优化：

- `opt-level = "z"`
- `lto = true`
- `codegen-units = 1`
- `strip = true`
- `panic = "abort"`
