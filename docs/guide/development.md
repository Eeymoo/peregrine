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

## Telemetry Development

Peregrine integrates anonymous GlitchTip (Sentry-protocol) telemetry: crash reports, startup stats, and key-path error reporting. The user-facing privacy explanation is in [Privacy & Telemetry](./privacy.md); the developer-facing registry of every report code is in [`REPORT_CODES.md`](./report-codes) at the repo root.

### Module layout

| Layer | File | Responsibility |
|---|---|---|
| Rust backend | `src-tauri/src/telemetry.rs` | install_id lifecycle, pending storage, event anonymization, panic hook, `report_code` constants, startup / `safe_try!` error outlets |
| Frontend | `src/lib/telemetry.ts` | `REPORT_CODES` constants, `initTelemetry`, `captureFrontendError`, pending / authorize Tauri command wrappers |
| Build glue | `src-tauri/build.rs` | DSN injection from `.env.development`, emits `peregrine_disable_telemetry` cfg |

### DSN environment variables

DSN is injected at **build time**, never read at runtime from disk:

| Build mode | Env var | Source |
|---|---|---|
| `dev` / debug | `GLITCHTIP_DSN_TEST` (Rust) / `VITE_GLITCHTIP_DSN_TEST` (frontend) | Repo-root `.env.development` (gitignored, copy from `.env.development.example` if present) or external env |
| `release` | `GLITCHTIP_DSN` (Rust) / `VITE_GLITCHTIP_DSN` (frontend) | CI GitHub Secrets `GLITCHTIP_DSN` / `VITE_GLITCHTIP_DSN` |
| snapshot | same as release but TEST project | `GLITCHTIP_DSN_TEST` mapped to `GLITCHTIP_DSN` in the workflow |

`src-tauri/build.rs` parses `.env.development` and propagates `GLITCHTIP_DSN_TEST` to the compile-time environment so `option_env!("GLITCHTIP_DSN_TEST")` in `telemetry.rs::dsn()` can read it. The frontend reads it through Vite's `import.meta.env`.

**Local debugging without a DSN**: if neither `.env.development` nor the external env sets the variable, both Rust `dsn()` and the frontend `TELEMETRY_DSN_AVAILABLE` return `None`/`false`, the SDK stays uninitialized, and the app produces **zero network requests**. The telemetry UI is hidden in this case. This is the default state for fresh clones.

> Note: editing `.env.development` may not trigger a recompile of Rust sources that use `option_env!`. Run `cargo clean -p peregrine-tauri` (or `touch` a Rust source file) before rebuilding to pick up the new DSN.

### `PEREGRINE_DISABLE_TELEMETRY` (compile-time disable)

For builds that must contain **no reporting code whatsoever**, set the env var to any value before building:

```bash
PEREGRINE_DISABLE_TELEMETRY=1 cargo build --release
PEREGRINE_DISABLE_TELEMETRY=1 npx tauri build
```

`build.rs` emits the `peregrine_disable_telemetry` cfg; the whole `telemetry` module then compiles to no-op stubs that preserve API signatures but do no IO and no networking. No panic hook is registered.

### `safe_try!` convention

`safe_try!($expr, $code)` (defined in `src-tauri/src/lib.rs`) wraps any `Result`-returning call:

- **Ok** → pass through, no report.
- **Err** → capture function name + caller file:line + sanitized message (`anonymize_text` runs on the message), report via `telemetry::report_safe_try_error`:
  - SDK active → sentry event (Error level, tags `code` / `event_type=error` / `priority=p2` / `function` / `location`).
  - SDK inactive → falls back to local pending storage, zero network.
- The original `Err` is **returned as-is** so the caller can degrade gracefully.

**Use sparingly — only on key paths**: file IO, render entry points, window bridge, external calls. Do **not** wrap every method; that dilutes signal and floods the issue list. The Code passed in **must** be registered in `REPORT_CODES.md` and the `report_code` module first.

Currently wired `safe_try!` codes: `PGR-2101` (config IO) at four sites in `lib.rs`, `PGR-4101` (overlay render) at the render loop in `overlay.rs`. Reserved-but-unwired codes (`PGR-2102`, `PGR-2401`, `PGR-2501`, `PGR-4102`, `PGR-4201`, `PGR-4202`, `PGR-51xx`) are documented as such in `REPORT_CODES.md`.

### Code registration governance

Before adding a new report point:

1. Pick a free code in the appropriate number range (see `REPORT_CODES.md` table of ranges).
2. Add the constant to `report_code` (Rust) or `REPORT_CODES` (frontend) **and** a row to `REPORT_CODES.md` — both in the same PR.
3. Only then write the `safe_try!` / `capture_message` / `captureFrontendError` call site.
4. Mark the row's "接线状态" as ✅ 已接线 with the actual trigger point (function / file).

Hardcoded codes that bypass this registry are not allowed in PRs. Codes are stable once shipped — never renumber or reuse a code.

### Frontend wiring

The frontend initializes Sentry once on startup (`src/main.tsx`):

```ts
const telemetryEnabled = config.settings.telemetry_enabled === true;
initTelemetry(telemetryEnabled);
```

`initTelemetry` is a no-op unless both the flag is `true` **and** `TELEMETRY_DSN_AVAILABLE` is `true`. The same `beforeSend` anonymization as the Rust side runs in-browser.

Errors are funneled through `captureFrontendError(code, error, tags?)`:

- SDK initialized → sentry event with the `code` tag.
- SDK not initialized → falls back to the `store_pending_report` Tauri command so the record still lands on disk.

The first-launch consent dialog (`ConfigApp.tsx`) and the error page's one-time "upload error report" button (`ErrorBoundary.tsx`) are the only places that change `telemetry_enabled` or call `authorizeUploadAll`. Treat these as the user-facing surface; everything else is fire-and-forget diagnostics.
