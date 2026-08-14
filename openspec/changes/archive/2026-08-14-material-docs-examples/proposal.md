# 物料创作文档与示例

> **取代说明**：本 change 已由 `restore-dynamic-material`（2026-08-14）收编并取代——与本 change 范围重叠的未竟事项已并入该 change 的 What Changes，其余宣告放弃。随该 change 归档，详见 `openspec/changes/restore-dynamic-material/proposal.md` 头部取代关系说明。

## Why

`four-layer-customization` 的物料运行时、12 份内置物料与图层 UI 已落地，但**用户创作侧的最后一公里缺失**：没有创作指南文档、没有示例物料、README 仍是旧样式清单。用户即使知道物料是 `.rhai` 脚本，也没有"照抄 → 修改 → 验证"的入门路径。该部分在原 change 的 tasks.md §19 中被错误勾选为已完成，现拆出独立交付。

## What Changes

### 新增

- **`docs/guide/material-scripting.md`**：物料脚本创作指南——按 design.md 决策 11 的五步流程（选图元 → 定布局 → 抽参数 → 声明 defaults/schema → 验证）组织，覆盖三函数约定、动态输入 API（`time_ms` / `mouse_pos` / `key_down` / `rand`）、沙箱限制说明、完整示例。
- **`docs/guide/layers.md`**：图层管理使用说明——图层层概念、叠加顺序、变换 / 样式 / 可见性、与 Profile 的关系。
- **`docs/.vitepress/config.mts`**：注册上述两篇文档的侧边栏条目（含 zh-cn 镜像，如适用）。
- **`crates/material/examples/`**：3 个示例物料脚本——静态（简易十字变体）、时间动态（时钟）、输入动态（鼠标跟随 / 键盘响应）。每个示例 MUST 能被 `Material::load` 加载并成功求值。
- **`time.rhai` 归位**：当前误置于 `crates/material/builtin/` 的动态时钟物料，评估后移至 `examples/`（避免其占用内置物料命名空间），或明确保留为内置并补充文档说明。

### 修改

- **`README.md` / `README.zh-cn.md`**：第 59 行的旧样式清单更新为四层架构描述（元素 / 物料 / 图层 / 配置 + 用户可编程物料）。

### 删除

无。

## Capabilities

### New Capabilities

- `material-authoring-guide`: 物料创作文档（创作指南 + 图层使用说明 + 侧边栏注册）、示例物料库、README 同步。

### Modified Capabilities

（无：`four-layer-customization` 尚未归档，`openspec/specs/` 下无既有 capability 需要修改 delta；本 change 交付的是其 material-runtime spec 中"创作示例与文档可用"场景对应的交付物。）

## Impact

### 代码影响面

| 模块 | 影响等级 | 改动概要 |
|---|---|---|
| `docs/guide/` | 低 | 新增两篇文档 |
| `docs/.vitepress/config.mts` | 低 | 侧边栏注册 |
| `crates/material/examples/` | 低 | 新增 3 个示例 `.rhai`（不进二进制，纯文件） |
| `crates/material/builtin/` | 低 | `time.rhai` 归位评估（若移出，`BUILTIN_MATERIALS` 列表同步调整） |
| `README.md` / `README.zh-cn.md` | 低 | 样式清单描述更新 |

### 依赖变更

无。

### 向后兼容

- 纯文档 / 示例交付，不改变任何运行时行为。
- 若 `time.rhai` 从内置移出：引用 `builtin.time` 的既有图层会失去物料（求值失败跳过该图层，不崩溃）。迁移策略：检查默认配置是否引用，若无引用则直接移出；若有引用则保留为内置。

### 发布版本

随下一 alpha（`v0.2.0-alpha.1`）或 stable（`v0.2.1`）发布，无独立版本要求。
