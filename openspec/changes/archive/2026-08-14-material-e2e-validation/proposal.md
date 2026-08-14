# 物料运行时端到端验证与质量基线

> **取代说明**：本 change 已由 `restore-dynamic-material`（2026-08-14）收编并取代——与本 change 范围重叠的未竟事项已并入该 change 的 What Changes，其余宣告放弃。随该 change 归档，详见 `openspec/changes/restore-dynamic-material/proposal.md` 头部取代关系说明。

## Why

`four-layer-customization` 的功能实现已完成（106 单元测试通过），但**发布 stable 所需的质量证据缺失**：迁移回归集成测试（tasks.md 7.10）从未创建却被勾选完成，§21 的七条端到端验证（真实配置迁移、性能、内存、体积、求值延迟、错误隔离、动态物料实效）全部未做。这些项彼此同质（都是验证而非新功能），拆为一个独立 change，作为 `v0.2.1` stable 发布的准入门槛。

## What Changes

### 新增

- **迁移回归集成测试** `crates/config/tests/migration_regression.rs`：每种旧 style 迁移后调用真实物料求值，与旧 `build_shapes` 输出逐元素、逐字段对比（13 个 style × 典型参数组合，含 `toilet_paper` alias）。
- **性能基线**：1080p / 多图层 / 60fps 下的帧时间与求值延迟基准（静态物料缓存命中 < 1µs、单图层单次求值 < 100µs、3 图层单帧 < 8ms）。
- **资源基线**：对比 v0.1.x 的内存增量（物料缓存 + Rhai engine < 10MB）与 release 二进制体积增量（< 500KB）。
- **长时间稳定性**：1080p / 5 图层 / 60fps 连续渲染 1 小时无明显掉帧（frame time < 16ms）。
- **真实配置迁移验证**：5 份覆盖 12 种样式的真实旧配置，手动确认迁移零视觉退化。
- **用户物料错误隔离验证**：语法错误 / 运行时异常 / 死循环 / 调用未注册函数四类场景均不崩溃。
- **动态物料实效验证**：时钟物料每秒更新、鼠标跟随延迟 < 50ms、键盘响应即时。

### 修改

- **`Cargo.toml`（crates/config）**：如需，`[dev-dependencies]` 增加 `peregrine_material` path 依赖以支持集成测试（允许 dev 依赖跨越"config 不依赖运行时"的边界，仅限测试）。

### 删除

无。

## Capabilities

### New Capabilities

- `material-quality-baseline`: 迁移视觉等价性回归、性能 / 内存 / 体积基线、错误隔离与动态物料的端到端验收标准。

### Modified Capabilities

（无：`four-layer-customization` 尚未归档，`openspec/specs/` 下无既有 capability；本 change 交付的是其 §21 端到端验证与 7.10 集成测试。）

## Impact

### 代码影响面

| 模块 | 影响等级 | 改动概要 |
|---|---|---|
| `crates/config/tests/` | 中 | 新增迁移回归集成测试（dev-dependency 引入 `peregrine_material`） |
| `crates/material` 或独立 bench | 低 | 求值延迟 / 缓存命中基准（可用 `cargo bench` 或简单计时测试） |
| `src-tauri` | 无 | 不改 |
| CI | 低 | 视情况将迁移回归测试纳入 ci.yml（已在 workspace 测试范围内则无需改） |

### 依赖变更

- **dev-only**：`crates/config` 的 `[dev-dependencies]` 可能新增 `peregrine_material`（path）。不进入 release 依赖图。

### 向后兼容

纯测试 / 验证交付，不改变运行时行为。若验证发现性能或视觉退化超标，另开修复 change，不在本 change 内改实现。

### 发布版本

本 change 是 `v0.2.1` stable 发布的**准入门槛**：全部验收通过前不发 stable。期间可继续发 alpha。
