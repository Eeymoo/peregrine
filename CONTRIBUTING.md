# Contributing Guide

Thank you for considering contributing to Peregrine! This document explains the process and guidelines for participating in the project.

---

## Quick Start

1. Fork this repository.
2. Create a feature branch: `git checkout -b feat/your-feature-name`.
3. Develop and test locally.
4. Make sure all tests pass: `cargo test`.
5. Submit a Pull Request.

---

## Branch Naming Conventions

| Prefix | Purpose |
| --- | --- |
| `feat/` | New feature (e.g. `feat/add-linux-support`) |
| `fix/` | Bug fix (e.g. `fix/overlay-crash`) |
| `refactor/` | Code refactoring (e.g. `refactor/render-pipeline`) |
| `docs/` | Documentation updates (e.g. `docs/contributing-guide`) |
| `ci/` | CI/CD configuration changes (e.g. `ci/add-macos-builder`) |
| `chore/` | Miscellaneous (dependency updates, build scripts, etc.) |

Please keep branch names short and descriptive.

---

## Commit Message Conventions

Follow the [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <short description>

<detailed description (optional)>
```

### type (required)

| Type | Description |
| --- | --- |
| `feat` | New feature |
| `fix` | Bug fix |
| `refactor` | Refactoring without functional changes |
| `docs` | Documentation |
| `ci` | CI/CD |
| `chore` | Miscellaneous |
| `test` | Tests |

### scope (optional)

Indicates the affected scope, for example `config`, `renderer`, `overlay`, `settings-ui`, `ci`, etc.

### Examples

```
feat(overlay): add softbuffer pixel buffer approach
fix(renderer): fix opacity failure under sRGB
docs: add contributing guide and startup instructions
ci(release): only build and release Windows x86/x86_64/ARM64
```

As the project is moving toward internationalization, commit message bodies should be written in **English**. `type` and `scope` remain in English.

---

## Development Workflow

1. Create a feature branch from `main`.
2. Develop on the branch, **keeping commits reasonably sized** (don't stuff too many changes into one commit).
3. Before submitting, run:
   ```bash
   cargo test          # make sure tests pass
   cargo fmt           # keep code formatting consistent
   cargo clippy        # check common issues
   ```
4. Push the branch and create a Pull Request to `main`.
5. PR titles use the same style as commit messages (e.g. `feat(overlay): add PNG image support`).
6. Describe the changes and how to test them in the PR description.
7. Wait for Code Review and revise based on feedback.
8. The branch will be deleted after merging.

---

## Code Style

- Follow standard Rust style (`cargo fmt` default configuration).
- As the project is internationalized, please write **English** documentation comments (`///`) for public items, and use `//!` at the top of modules to describe responsibilities.
- Error handling: use `ConfigError` defined by `thiserror` at the library level; do not `panic`/`unwrap` in libraries.
- Logging uses `tracing`; do not add new `println!`/`eprintln!`.
- When adding new fields, be sure to add `#[serde(default)]` to maintain backward compatibility.
- Enum serialization should uniformly use `#[serde(rename_all = "snake_case")]`.

---

## Testing Requirements

- New features should include corresponding unit tests.
- Validation rule changes in `schema.rs` should be updated with tests.
- After configuration structure changes, run at least `cargo test -p peregrine_config`.
- Tests involving tokio use `#[tokio::test]`; tests involving filesystem events use `#[tokio::test(flavor = "multi_thread")]`.

---

## Pull Request Process

1. **Create PR**: submit to the `main` branch of this repository.
2. **CI Checks**: GitHub Actions will automatically run tests and lint after submission; all must pass.
3. **Code Review**: requires approval from at least one maintainer before merging.
4. **Merge Method**: use **Squash & Merge** to compress multiple commits on the branch into one commit when merging into `main`.

---

## Reporting Issues

When submitting an Issue, please include the following information if possible:

- Peregrine version (tag or commit hash)
- Windows version (e.g. Windows 10 / 11)
- Reproduction steps
- Expected behavior and actual behavior
- Logs or screenshots (if available)

---

Thank you again for your contribution!
