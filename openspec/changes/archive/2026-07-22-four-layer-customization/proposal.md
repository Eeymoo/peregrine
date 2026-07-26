# 四层可定制化架构重构

## Why

当前 `Crosshair` 是个 50+ 字段的封闭枚举（`crates/config/src/schema.rs:151-392`），新增样式必须同步修改 5 处代码，且 Rust（`crates/peregrine/src/shapes.rs::build_shapes`）与 TypeScript（`src/lib/shapes.ts::buildShapes`）两份几何实现必须手写一致——这是反复出现 WYSIWYG bug 的根源，也阻塞了用户自定义样式。

Peregrine 需要从"开发者定义、用户选择"的封闭系统，演进为"用户可编程"的开放系统：让用户能用轻量脚本自由组合图元、响应鼠标/键盘/时间，并把多个组件叠加成图层化配置。

## What Changes

### 新增

- **元素层（Element）**：将 `crates/peregrine/src/shapes.rs::Shape` 升格为正式的、可序列化的配置类型，新增 `Text`、`Polygon`、`Image`、`Line` 等基础图元。
- **物料层（Material）**：引入 Rhai 嵌入式脚本作为物料运行时，新建 `crates/material` crate。物料脚本定义"参数 → Element 列表"的纯映射，并声明参数 schema 供 UI 自动生成控件。
- **图层层（Layer）**：`Profile` 从单个 `Crosshair` 改为 `Vec<Layer>`，每个图层引用一个物料实例并携带参数、变换（位移/缩放/旋转）、样式（颜色/不透明度/混合）、可见性。
- **动态输入 API**：Rhai 脚本可访问 `time_ms()`、`mouse_pos()`、`key_down(code)`、`rng()` 等 host function，支持实现"动态响应"物料。
- **内置物料库**：12 种现有 `CrosshairStyle` 全部翻译为内置 `.rhai` 物料，通过 `include_str!` 嵌入二进制。
- **用户物料库**：物料脚本存放在 `%APPDATA%/Peregrine/materials/*.rhai`，启动时扫描合并；同名时用户物料优先级高于内置。
- **预览 IPC**：新增 Tauri command `build_shapes(profile_id, screen_size)`，前端 Preview 改为通过 IPC 获取 Element 列表后用 Canvas 渲染。
- **物料缓存**：`(material_id, params_hash, screen_hash)` 作为 key 的 LRU 缓存，避免每帧重算 Rhai 脚本。
- **物料参数 schema**：Rhai 脚本通过 `fn schema()` 导出参数元数据（label、min、max、step、widget 类型），前端据此动态生成控件。

### 修改

- **`crates/config/src/schema.rs`**：新增 `Element`、`Layer`、`MaterialRef`、`Transform2D`、`LayerStyle` 类型；`Profile.crosshair` 字段替换为 `Profile.layers: Vec<Layer>`。**BREAKING**（运行时自动迁移，用户无感）。
- **`crates/peregrine/src/shapes.rs`**：`build_shapes` 不再按 `CrosshairStyle` 分支，改为遍历图层调用 `peregrine_material::evaluate`。
- **`crates/peregrine/src/overlay_renderer.rs`**：渲染循环从"单一 crosshair"改为"遍历图层"，每层独立应用 transform/style 后光栅化。
- **`src/lib/shapes.ts`**：删除手写几何函数，改为调用 Tauri IPC `build_shapes` 获取 Element 列表。
- **`src/components/Preview.tsx`**：保留 Canvas 渲染逻辑，数据源改为 IPC 结果。
- **`src/components/StyleFields.tsx`**：改为根据物料 `schema()` 动态生成参数控件。
- **`src/types/config.ts`**：同步 Rust 侧新 schema。
- **`src-tauri/src/lib.rs`**：所有 commands（`save_config`、`set_crosshair_color` 等）从操作 `crosshair` 改为操作 `layers`。

### 删除

- `CrosshairStyle` 枚举及其 12 个变体（迁移到 Rhai 物料后不再需要）。
- `Crosshair` 结构体的全部样式专用字段（`ring_radius_pct`、`arrow_*`、`grid_*`、`border_*`、`orb_positions` 等）——这些参数由各自物料的 `defaults()` 提供。
- `crates/peregrine/src/shapes.rs::build_shapes` 中按 style 的巨型 match 块（~500 行）。
- `src/lib/shapes.ts::buildShapes` 全部几何逻辑（~280 行）。
- `RandomOrbMode`、`BorderFrameStyle`、`RingStyle`、`GridAlignment`、`Anchor`、`OrbPosition` 等样式专用枚举（迁移为各物料的参数枚举）。

## Capabilities

### New Capabilities

- `element-primitives`: 基础图元（Rect / Circle / Triangle / Text / Image / Line / Polygon 等）的定义、序列化、光栅化。
- `material-runtime`: Rhai 脚本运行时；物料定义/加载/缓存/沙箱；参数 schema 自动生成。
- `material-dynamic-input`: 物料脚本的动态输入 host function（时间/鼠标/键盘/随机数）。
- `layer-composition`: 多图层模型；图层变换、样式、可见性；图层叠加渲染顺序。
- `profile-migration`: 旧 `Crosshair` 配置到新 `layers` 配置的自动迁移。

### Modified Capabilities

（首次建立 spec-driven 文档，无既有 capability 需要修改 delta）

## Impact

### 代码影响面

| 模块 | 影响等级 | 改动概要 |
|---|---|---|
| `crates/config` | 高 | schema 重写；validator 重写；迁移逻辑新增 |
| `crates/material`（新建） | 高 | Rhai 集成；host function；缓存；内置物料嵌入 |
| `crates/peregrine` | 中 | `build_shapes` 改签名；`overlay_renderer` 改多图层遍历 |
| `src-tauri/src/lib.rs` | 高 | 几乎所有 commands 适配新 schema；新增 `build_shapes` command |
| `src-tauri/src/overlay.rs` | 中 | 渲染循环对接新图层模型 |
| `src/lib/shapes.ts` | 中（删除） | 全部几何逻辑迁移到后端 |
| `src/components/Preview.tsx` | 中 | 数据源改为 IPC 异步 |
| `src/components/StyleFields.tsx` | 高 | 重写为根据 schema 动态生成控件 |
| `src/types/config.ts` | 高 | 类型同步 |
| 测试 | 中 | `crates/config` 现有 30+ 测试需要适配新 schema；`crates/material` 新增物料求值测试 |

### 依赖变更

- **新增**：`rhai = "1.19"`（workspace.dependencies）；`rhai-fs = "0.2"`（用户物料加载，可选）。
- **可能新增**：`ahash = "0.8"`（物料缓存 key 哈希）。
- **不变**：`winit`、`softbuffer`、`png`、`serde`、`tokio`、`tauri`。

### 架构边界

- `crates/config`：保持纯数据/校验，不引入 Rhai 依赖。新增 `Element`、`Layer` 等数据类型。
- `crates/material`（新建）：依赖 `peregrine_config` 与 `rhai`；提供物料求值 API。
- `crates/peregrine`：依赖 `peregrine_material`；保持光栅化职责。
- `src-tauri`：依赖 `peregrine_material`；提供 IPC。

### 向后兼容

- **用户配置文件**：检测旧格式（含 `crosshair.style` 且无 `layers`）自动迁移为单图层，备份为 `config.json.legacy.bak`，对用户透明。
- **行为不变**：迁移后视觉效果与旧版完全一致（通过单测保证 12 种内置物料的几何输出与旧 `build_shapes` 逐像素一致）。
- **配置目录**：`%APPDATA%/Peregrine/config.json` 路径不变；新增 `%APPDATA%/Peregrine/materials/` 目录。

### 发布版本

- 目标版本：`v0.2.0-alpha.0`（尝鲜版，偶数版本号），稳定后发 `v0.2.1`。
- 对应 `dev` 分支开发，稳定后合并 `main`。

### 性能预期

- overlay 60fps 渲染：物料缓存命中时零计算开销；参数变化时单次 Rhai 求值 < 100µs（Rhai 文档数据）。
- 前端预览：IPC 单次往返约 1-5ms，可接受。
- 二进制体积：Rhai 约 +500KB（release + lto 后），可接受。
