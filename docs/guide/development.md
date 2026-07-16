# Development & Build

## Requirements

- Rust 1.85 or later (edition 2024)
- Node.js 20 or later (for frontend builds)
- Windows SDK (for Win32 API and the `windows` crate)
- Cargo

## Clone the Repository

```bash
git clone https://github.com/eeymoo/peregrine.git
cd peregrine
```

## Build

```bash
# Install frontend dependencies
npm install

# Debug build
cargo build

# Release build (smaller binary, better performance)
cargo build --release

# Run the Tauri development version (with hot reload)
npx tauri dev

# Build the Tauri release installer
npx tauri build
```

## Testing

```bash
# Run all tests
cargo test

# Run only the config crate tests
cargo test -p peregrine_config
```

## Linting

```bash
cargo fmt
cargo clippy -p peregrine_config -- -D warnings
```

## Release Artifacts

The release artifacts produced by `npx tauri build` are located under `src-tauri/target/release/`. The MSI installer is located at `src-tauri/target/release/bundle/msi/`.

Release builds are optimized for size and performance:

- `opt-level = "z"`
- `lto = true`
- `codegen-units = 1`
- `strip = true`
- `panic = "abort"`
