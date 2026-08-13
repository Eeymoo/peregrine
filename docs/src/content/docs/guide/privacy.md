---
title: "Privacy & Telemetry"
---

Peregrine ships with an **anonymous, opt-in** telemetry feature that helps us diagnose crashes and prioritize fixes. This page explains exactly what is collected, what is *not* collected, how to turn it off, and how self-build users can compile it out entirely.

> TL;DR: We never collect IP addresses, usernames, machine names, device identifiers, or file paths containing your username. Telemetry only fires for **startup stats, crashes, and a handful of key error paths** — never for content, configuration values, or browsing behavior.

## What is collected

When telemetry is enabled **and** the binary ships with a DSN injected at build time (official builds), the following is sent to our self-hosted GlitchTip instance:

| Event | Level | When | Data attached |
|---|---|---|---|
| **App startup** | Info | Once per process launch | `code=PGR-0001`, `install_id`, `version`, `os`, `arch` |
| **Rust panic / crash** | Error | Any `panic!` (captured by a custom panic hook, persisted locally first) | `code=PGR-1001`, `install_id`, `version`, sanitized stacktrace / message |
| **Key-path error** | Error | `safe_try!` macro catches IO / render / bridge failures | `code=PGR-2xxx/4xxx`, function name, file:line, sanitized message |
| **Frontend error** | Error | React `ErrorBoundary`, `window.onerror`, `unhandledrejection` | `code=PGR-3xxx`, sanitized message & stack |

Each event carries a fixed set of tags:

- `install_id` — a random UUID v4 generated on first launch and stored in `<app_data_dir>/install_id`. It is **uncorrelated** with any real identity and can be regenerated at any time by deleting that file.
- `version` — the application version string (e.g. `0.2.0`).
- `os` / `arch` — `std::env::consts::OS` / `ARCH` (e.g. `windows` / `x86_64`).
- `code` — a stable identifier from the [REPORT_CODES registry](./report-codes) indicating the event class.
- `event_type` — `startup` / `crash` / `error`.
- `priority` — `p1` (crash) / `p2` (error) / `p3` (startup).

Crash reports additionally include the Rust / JS stacktrace and the panic message (after sanitization, see below).

## What is NOT collected

We **do not** collect any of the following:

- ❌ IP addresses — events are scrubbed of `user`, `server_name`, and `request` fields before leaving the device (sentry `before_send` hook).
- ❌ Usernames — Windows path components such as `C:\Users\<name>`, `/Users/<name>`, `/home/<name>` are replaced with `C:\Users\{user}` / `/Users/{user}` / `/home/{user}` in messages, stacktraces, and `abs_path` frames.
- ❌ Machine name / hostname / device IDs (no `server_name`, no SMB/NetBIOS name).
- ❌ Configuration contents — never the crosshair style, layer params, profile names, target window titles, or hotkey bindings.
- ❌ Game titles or window titles (the `target_window` field is a user-setting and is **not** transmitted).
- ❌ Screenshots, clipboard, file contents.
- ❌ Browsing history, input events, mouse coordinates (dynamic-input APIs exist for material scripts but are currently soft-disabled and never sent off-device).

## Data sanitization

Both the Rust and frontend Sentry clients apply an identical `anonymize_event` / `beforeSend` hook:

1. Delete `event.user` (drops IP, username if any).
2. Delete `event.server_name` (drops hostname).
3. Delete `event.request` (drops URL / headers).
4. Replace path-username patterns in `message`, `exception.value`, and each stacktrace frame's `filename` / `abs_path`.

The replacement patterns cover Windows (`C:\Users\x`), macOS (`/Users/x`), and Linux (`/home/x`) home directories.

## Consent and toggle

Telemetry follows a **first-launch consent** model:

- On the first launch the settings window shows a one-time dialog asking whether to enable anonymous reporting. Your choice (`true` / `false`) is persisted to `config.settings.telemetry_enabled`.
- If the field is missing (`null`), the SDK **does not initialize** — silence is treated as **not authorized**, not as consent.
- The toggle lives in **Settings → Telemetry** ("Anonymous crash reports & usage data"). Changing it requires a restart; a "pending restart" badge reminds you.
- The error page also offers a **one-time "upload error report"** button: tapping it initializes the SDK just long enough to flush locally-pending records, then tears it down. It does **not** flip the persistent switch.

## Local pending storage

When telemetry is off (or no DSN was injected), crashes and key-path errors are still written to disk so you can review or upload them later:

- Path: `<app_data_dir>/pending/*.json` (one file per record, atomic writes).
- Cap: 5 MB total — oldest records are rotated out first.
- Records contain `ts`, `version`, `install_id`, `code`, sanitized `message` — nothing else.

You can see the pending count on the error page and choose to upload (or simply delete the files).

## Compile-time disable (`PEREGRINE_DISABLE_TELEMETRY`)

Self-build users who want zero telemetry code in the binary can disable it at compile time:

```bash
# Set the env var before building (any value works)
PEREGRINE_DISABLE_TELEMETRY=1 cargo build --release
# Or for a Tauri build:
PEREGRINE_DISABLE_TELEMETRY=1 npx tauri build
```

When set, `src-tauri/build.rs` emits the `peregrine_disable_telemetry` cfg, and the entire `telemetry` module compiles to no-op stubs: all public APIs keep their signatures but contain no IO, no network, no panic hook. The resulting binary contains no reporting code paths whatsoever.

## Where the data goes

- Our GlitchTip instance is self-hosted (Sentry-protocol compatible).
- Official release builds report to the **production** project; official development/snapshot builds report to the **test** project. Self-builds without a DSN produce **zero network traffic**.

## Summary

| Question | Answer |
|---|---|
| Can you see my identity? | No. IP, username, machine name, device ID are all stripped or never collected. |
| Can you see my config / game? | No. Only crash/error diagnostics and coarse environment tags. |
| Is it on by default? | No. First launch asks; missing consent is treated as off. |
| Can I turn it off? | Yes — Settings → Telemetry, any time (restart required). |
| Can I build without it? | Yes — `PEREGRINE_DISABLE_TELEMETRY=1` strips all reporting code at compile time. |

For the developer-facing registry of every report code, see [`REPORT_CODES.md`](./report-codes) in the repository root.
