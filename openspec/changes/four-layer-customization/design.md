# 四层可定制化架构 - 技术设计

## Context

### 当前状态

Peregrine 当前的配置模型围绕 `Crosshair` 这个"上帝结构"展开（`crates/config/src/schema.rs:151-279`，50+ 字段）。配合一个封闭的 `CrosshairStyle` 枚举（12 个变体，`schema.rs:364-392`），所有几何计算集中在 `crates/peregrine/src/shapes.rs::build_shapes` 的巨型 match 块中（约 500 行）。

更严重的是，前端预览（`src/lib/shapes.ts::buildShapes`，280 行 TypeScript）必须**逐分支手写复制**Rust 的几何算法（包括一个完整的 `SimpleRng` LCG 随机数生成器、`OrbPosition` 位运算、`SimpleRng` 的 u64 wrapping_add 语义等）。这套双实现是反复出现 WYSIWYG bug 的根源。

### 约束

- **平台目标**：Windows 为主，overlay 透明 / 点击穿透 / 窗口跟随依赖 Win32 API（`crates/peregrine/src/platform/windows.rs`）。但 `crates/config` MUST 保持零平台依赖。
- **架构边界**（AGENTS.md 明文规定）：`peregrine_config` 不得依赖任何 UI / GPU / window 代码；渲染逻辑在 `peregrine` 共享库；Tauri 入口在 `src-tauri`。
- **并发模型**：跨 tokio / winit 线程的配置快照使用 `std::sync::Mutex`（非 `tokio::sync::Mutex`，避免 `blocking_lock` panic）。`ConfigSnapshot = Arc<AppConfig>`。
- **配置持久化**：原子写（临时文件 + rename），校验前置（`validate()` 总在 `save()` 前调用），损坏文件自动备份为 `.bak` 并回退默认。
- **序列化兼容**：新增字段 MUST 用 `#[serde(default)]`；枚举统一 `#[serde(rename_all = "snake_case")]`。
- **语言 / 注释**：所有公开项 MUST 有简体中文 doc comment。
- **Rust 版本**：edition 2024，rust-version 1.85。

### 相关利益方

- 现有用户：期望升级后行为零变化（迁移对用户透明）。
- 高级用户：希望能自定义样式（写自己的物料）。
- 开发者：希望减少双实现、降低新增样式的边际成本。

## Goals / Non-Goals

### Goals

1. **彻底消除 Rust/TS 几何双实现**：前端预览改为通过 IPC 调用后端同一份物料求值逻辑。
2. **用户可编程样式**：用户能用 Rhai 脚本定义任意物料，无需修改 Rust 代码。
3. **多图层叠加**：Profile 支持任意多个图层组合，每层独立可见性 / 样式 / 变换。
4. **动态响应物料**：物料可读取时间 / 鼠标 / 键盘 / 随机数，实现动态效果。
5. **12 种现有样式零视觉退化**：迁移后像素级一致。
6. **架构边界清晰**：新增 `crates/material` 不破坏 `peregrine_config` 纯数据原则。

### Non-Goals

1. **物料市场 / 在线分享**：不实现物料脚本的在线上传 / 下载 / 商店。物料来源仅限本地文件。
2. **撤销 / 重做（Undo / Redo）**：图层操作不实现历史栈，后续版本再考虑。
3. **图层蒙版 / 矢量路径编辑器**：不提供 Photoshop 级别的图层编辑能力。图层变换仅限平移 / 缩放 / 旋转。
4. **GPU 渲染**：保持 softbuffer CPU 光栅化；wgpu 方案不在本次范围。
5. **跨平台 overlay**：依然 Windows-only。Rhai 物料求值本身跨平台，但 overlay 窗口跟随仍依赖 Win32 API。
6. **Tauri → rhai-wasm 前端直跑**：本期预览走 IPC；前端运行 Rhai 留待未来 rhai-wasm 成熟后再上。
7. **游戏手柄 / 音频电平等扩展动态输入**：架构留好扩展点但本期不实现。

## Decisions

### 决策 1：新增 `crates/material` crate，作为 Rhai 运行时层

**选择**：新建独立 crate `peregrine_material`，依赖 `peregrine_config`（消费 `Element` 类型）+ `rhai`。

**理由**：
- 严守 AGENTS.md 架构边界：`peregrine_config` 保持纯数据 / 校验 / 持久化职责，不引入脚本运行时。
- Rhai 是新依赖，独立成 crate 便于未来替换（如换 deno_core / boa_engine）。
- 物料求值是纯函数（输入 → Element 列表），天然适合独立测试。
- 前端未来若引入 `rhai-wasm`，可直接基于 `peregrine_material` 的 wasm 构建产物。

**备选方案与拒绝理由**：
- *放进 `peregrine_config`*：违反 AGENTS.md 明文约束。
- *放进 `peregrine` 共享库*：`peregrine` 已含 winit/softbuffer，职责混杂；且 `peregrine` 当前不被 `src-tauri` 以外的消费者使用，新增依赖关系不优雅。

**Crate 关系图**：
```
+--------------------+
| peregrine_config   |  (纯数据：Element/Layer/Profile schema + validate + migrate + persist)
+---------+----------+
          |
          v
+--------------------+     +-------------------+
| peregrine_material | <-- | peregrine         |  (光栅化 + winit)
| (Rhai runtime)     |     | (build_shapes 改  |
+---------+----------+     |  为调用 material) |
          |                +---------+---------+
          |                          |
          v                          v
+--------------------+     +-------------------+
| src-tauri          | <-- | frontend (IPC)    |
| (commands, build_  |     | (Preview)         |
|  shapes IPC)       |     +-------------------+
+--------------------+
```

### 决策 2：物料脚本采用 Rhai，遵循 `build/defaults/schema` 三函数约定

**选择**：Rhai 1.19+，物料脚本必须导出 3 个顶层函数：
```rhai
fn defaults() -> Map { #{...} }
fn schema() -> Array { [...] }
fn build(params, screen) -> Array { [...] }
```

**理由**：
- Rhai 语法接近 JS，用户上手成本低（与用户回答中"rhino 轻量化脚本"期望一致）。
- Rhai 无 GC、零外部依赖、编译进二进制 +500KB（可接受）。
- 沙箱原生支持（`set_max_operations` / `disable_module_resolution`）。
- `schema()` 驱动 UI 控件自动生成，避免为每个物料手写 React 控件。

**备选方案与拒绝理由**：
- *MLua Lua*：语法更小众；GC 带来的帧时间不稳。
- *boa_engine (JS)*：性能慢于 Rhai，体积更大。
- *deno_core*：依赖 V8，二进制爆炸（+30MB+），不可接受。
- *WASM (wasmtime)*：用户写 WASM 门槛过高。

### 决策 3：内置物料通过 `include_str!` 嵌入，用户物料覆盖

**选择**：
- 内置物料源码：`crates/material/builtin/*.rhai`（12 个文件），通过 `include_str!` 编译进二进制。
- 用户物料：`%APPDATA%/Peregrine/materials/<name>.rhai`，启动时扫描。
- 加载优先级：用户物料 > 内置物料（同名时）。

**理由**：
- 嵌入保证开箱即用，零文件 IO；首次安装无需解压物料库。
- 用户物料覆盖机制：高级用户可"覆盖" `builtin.cross` 行为而无需改源码。
- 物料注册表用 `HashMap<MaterialId, Arc<Material>>`，启动时构建一次。

**备选方案与拒绝理由**：
- *全部外部文件*：删除后程序不可用，破坏"零依赖默认配置"哲学。
- *只嵌入 + 无用户覆盖*：失去用户自定义能力。

### 决策 4：预览走 Tauri IPC（方案 C2），删除 `src/lib/shapes.ts`

**选择**：
- 新增 Tauri command `build_shapes(profile_id: String, screen: Rect)`，返回 `Vec<Element>`。
- 前端 Preview 组件每次参数变化时调用 IPC，把返回的 Element 列表画到 Canvas。
- **彻底删除** `src/lib/shapes.ts` 全部 280 行 TS 几何逻辑。

**理由**：
- 一次实现、真 WYSIWYG。任何视觉 bug 修复只改一处。
- IPC 延迟（1-5ms）对预览（用户调整滑块时的反馈延迟 50-100ms 是可接受的）影响小。
- 后端可做缓存（`(material_id, params_hash, screen_hash)` LRU），减少重复计算。

**备选方案与拒绝理由**：
- *rhai-wasm 前端直跑*：体积 +300KB，依赖 rhai-wasm 维护活跃度；留作未来优化。
- *保留内置物料 TS 手实现，用户物料走 IPC*：双实现问题没解决；用户物料预览体验差。

**IPC 节流策略**：前端对参数变化做 16ms 节流（60fps），避免拖动滑块时打爆 IPC。

### 决策 5：物料求值三段管线 `Profile → Layers → Elements`

**选择**：求值流程分三段：
1. **物料求值**：对每个图层，调用 `material.build(params, screen)` 得到 `Vec<Element>`（图层局部坐标）。
2. **变换应用**：把 `Vec<Element>` 按 `layer.transform` 平移 / 缩放 / 旋转。
3. **样式应用**：把变换后的 `Element` 按 `layer.style` 染色（color、opacity 相乘）。

**理由**：
- 物料脚本不感知变换 / 样式，保持"纯几何"职责单一。
- 变换在 Element 层面应用（而非光栅化层面），保证 Rust 与 TS（未来可能的前端直跑）行为一致。
- 染色延迟到光栅化前，允许未来支持更复杂的混合模式。

**伪代码**：
```rust
fn build_shapes(profile: &Profile, screen: Rect, ctx: &DynamicContext) -> Vec<Element> {
    let mut out = Vec::new();
    for layer in profile.layers.iter().filter(|l| l.visible) {
        let material = material_registry.get(&layer.material)?;
        let params = merge_defaults(material.defaults(), &layer.params);
        let raw = material.evaluate(&params, &screen, &ctx)?;  // Rhai 调用
        let transformed = apply_transform(raw, &layer.transform, &screen);
        let styled = apply_style(transformed, &layer.style);
        out.extend(styled);
    }
    out
}
```

### 决策 6：`DynamicContext` 聚合动态输入，物料求值时显式传入

**选择**：定义 `DynamicContext` 结构体：
```rust
pub struct DynamicContext {
    pub time_ms: u64,
    pub mouse_pos: (f32, f32),
    pub key_state: KeyState,         // 按键状态表
    pub rng_seed: u64,               // 派生自 (material_id, params_hash, frame_count)
    pub version: u64,                // 变化时递增，使缓存失效
}
```

物料求值时，把 `DynamicContext` 通过 Rhai `Scope` 或 closure 捕获注入 host function：
```rust
engine.register_fn("time_ms", || ctx.time_ms);
engine.register_fn("mouse_pos", || #[rhai_map]#{ x: ctx.mouse_pos.0, y: ctx.mouse_pos.1 });
engine.register_fn("key_down", |code: &str| ctx.key_state.is_down(code));
```

**理由**：
- 每次求值用独立的 `Engine` 编译产物 + 共享 `AST`，host function 通过 closure 捕获 ctx，避免全局可变状态。
- `version` 字段用于缓存失效：静态物料（不读动态输入）`version` 永远为 0，缓存永久有效；动态物料每帧 `version` 递增。
- **关键**：一个物料是否"依赖动态输入"由其脚本元数据声明（`fn is_dynamic() -> Bool`），静态物料（如 `builtin.cross`）声明 `false`，永远走缓存路径。

**RNG 策略**：Rhai 中 `rand()` 内部使用 `DynamicContext.rng_seed` 派生的子种子。对 `builtin.random_orb`：种子计算方式 MUST 与旧 `SimpleRng` 一致（`shapes.rs:291-296`），保证迁移后视觉等价。

### 决策 7：图层操作 Tauri commands 复用现有持久化 / 广播流程

**选择**：新增 commands：`add_layer` / `remove_layer` / `move_layer` / `duplicate_layer` / `update_layer` / `list_layers`。每个 command：
1. 锁定 `state.config`，克隆 `AppConfig`。
2. 修改 `active_profile_mut().layers`。
3. `config.validate()` → `state.storage.save()` → `state.notifier.update()` → 更新 `shared_config` → 发送 `OverlayCommand::UpdateConfig`。
4. `app.emit("peregrine:layers-changed", &new_layers)` 通知前端。

**理由**：完全复用 `update_preferences_inner`（`src-tauri/src/lib.rs:819-959`）的成熟模式，不引入新的同步路径。

### 决策 8：迁移逻辑放在 `crates/config`，使用真实物料库求值做回归测试

**选择**：新建 `crates/config/src/migration.rs`：
- `pub fn migrate_legacy_crosshair(crosshair: &LegacyCrosshair) -> Result<Layer>`
- `pub fn migrate_app_config(legacy: &serde_json::Value) -> Result<AppConfig>`
- 依赖一份静态的"样式 → 物料 id + params 映射"表（无需 Rhai，只做字段搬运）。

迁移测试策略：
- 单元测试用 `include_str!("../../../../crates/material/builtin/cross.rhai")` 加载真实物料，迁移后调用 `peregrine_material::evaluate` 求值，与旧 `build_shapes` 输出对比。
- 测试放在 `crates/config/tests/migration_regression.rs`（集成测试，允许依赖 `peregrine_material`）。

**理由**：
- 迁移逻辑属于配置层，放 `crates/config` 符合分层。
- 不依赖 Rhai 即可完成字段映射（迁移本身不调用物料）；视觉等价性由集成测试验证。

### 决策 9：Rh(i) 错误隔离，单个图层失败不阻塞整体渲染

**选择**：`build_shapes` 内对每个图层调用物料求值时，捕获错误并记录 warning，跳过该图层继续渲染下一个。

**理由**：
- 用户物料脚本可能存在 bug（除零、类型错误、超时）。
- overlay 60fps 渲染 MUST NOT 因单个图层错误阻塞。

### 决策 10：版本号策略

**选择**：
- 目标版本：`v0.2.0-alpha.0`（首个预览版，偶数 minor + prerelease）
- 稳定版：`v0.2.1`（奇数 patch，按 AGENTS.md 约定）

**理由**：架构重构属于 BREAKING change（内部数据模型），需要 alpha 阶段收集反馈。

## Risks / Trade-offs

| 风险 | 严重度 | 缓解措施 |
|---|---|---|
| **Rhai 求值性能**：动态物料每帧调用 60 次 Rhai，可能掉帧 | 中 | 静态物料缓存（永久有效）；动态物料设上限（如最多 5 个图层）；Rhai `set_optimization_level(Simple)` |
| **二进制体积增加**：Rhai 约 +500KB（release + lto 后） | 低 | 已在 release profile 启用 `opt-level="z"` + `lto=true` + `strip=true`，预计最终增量 200-400KB |
| **物料脚本安全**：恶意 Rhai 脚本死循环或调用危险 API | 中 | `set_max_operations(100_000)` + `set_max_call_levels(64)` + `disable_module_resolution()` + 仅注册白名单 host function |
| **键盘状态隐私**：物料可读按键状态可能被滥用为键盘记录器 | 中 | host function 仅返回 Bool（按下/未按下），不返回事件序列；日志禁止打印按键代码；文档明确警告 |
| **前端预览延迟**：IPC 往返延迟在低端机上可能 > 10ms | 低 | 16ms 节流；后端 LRU 缓存；未来可换 rhai-wasm |
| **迁移失败导致用户配置丢失** | 高 | 三重保护：迁移前备份 `.legacy.bak`；迁移异常备份 `.legacy.bak.error`；最终回退默认配置 |
| **视觉退化**：12 种内置物料 Rhai 实现与旧 Rust 实现细微差异 | 高 | 完整回归测试（13 个用例覆盖全部样式 + alias），逐元素对比；特别是 `RandomOrb` 的 LCG 种子一致性 |
| **Rhai Map 字段顺序不稳定**：物料返回 `#{}` 的字段迭代顺序可能变化 | 低 | 物料求值结果只用 Array（Element 列表），不用 Map 迭代 |
| **`CustomImage` 物料的特殊性**：需要 PNG 加载/缓存 | 低 | 让 `Element::Image` 成为正式图元，渲染器侧仍用 `ensure_image_loaded` 缓存逻辑 |
| **Tauri IPC 大对象序列化开销**：返回大量 Element 时序列化耗时长 | 低 | 实际典型 Element 数 < 100，序列化 < 1ms；如未来需要可换 bincode 二进制 IPC |
| **Rhai 与 `SimpleRng` 一致性**：Rhai 中实现 LCG 必须与 `shapes.rs:727-745` 完全一致 | 中 | 在 Rhai host function 侧暴露 `rand()` / `rand_seed(seed)`，内部直接调用同一份 `SimpleRng` Rust 实现；不允许在 Rhai 中重新实现 RNG |

## Migration Plan

### 开发阶段（`dev` 分支）

1. **Step 1**：新建 `crates/material`，集成 Rhai，定义 `Material` / `MaterialRuntime` / `DynamicContext` 类型骨架。
2. **Step 2**：编写 1 个内置物料 `builtin/cross.rhai`，完成求值 + 测试。
3. **Step 3**：扩展 `peregrine_config::schema` 新增 `Element` / `Layer` / `MaterialRef` / `Transform2D` / `LayerStyle` 类型。
4. **Step 4**：迁移剩余 11 种内置物料（每种对照旧 `shapes.rs` 分支）。
5. **Step 5**：实现 `crates/config/src/migration.rs`，编写 13 个迁移单元测试。
6. **Step 6**：改造 `crates/peregrine/src/shapes.rs::build_shapes`，改为遍历图层调用物料求值。
7. **Step 7**：改造 `crates/peregrine/src/overlay_renderer.rs`，支持多图层渲染。
8. **Step 8**：在 `src-tauri` 新增 `build_shapes` IPC command + 图层操作 commands。
9. **Step 9**：改造前端：删除 `src/lib/shapes.ts`，Preview 改为 IPC；新增图层管理 UI；StyleFields 改为 schema 驱动。
10. **Step 10**：端到端测试 + 性能基准 + 文档。

### 发布阶段

- **Alpha**：`v0.2.0-alpha.0` 发到 prerelease 通道，收集反馈。
- **稳定**：1-2 周 alpha 后发布 `v0.2.1` 到 stable。

### 回滚策略

- 保留旧 `build_shapes` 代码到 `crates/peregrine/src/legacy_shapes.rs`（标 `#[deprecated]`），稳定版前可一键切换。
- 用户配置的 `.legacy.bak` 在用户目录保留，用户可手动还原。

## Open Questions

1. **Rhai AST 复用 vs 每次新建 Engine**：是否对所有物料预编译 AST 并缓存？还是要每次求值都解析？需要 benchmark 决定。（预期：预编译 AST，每次求值用 `Engine::call_fn` + 共享 `Scope`。）

2. **图层变换后图元是否 clamp 到屏幕边界**：超出屏幕的图元是否光栅化（被裁剪）？预期：不 clamp，渲染器自然裁剪。

3. **`update_layer` 的 patch 粒度**：整个 `params` 替换 vs 字段级更新？预期：字段级，每个参数变更触发一次 IPC（前端做节流）。

4. **物料脚本本地化**：`schema()` 返回的 label 是固定中文还是支持 i18n？预期：本期固定中文；未来物料 schema 支持 `label_zh: "尺寸", label_en: "Size"` 双字段。

5. **用户物料的"物料市场"格式兼容性**：当前设计是否与未来可能的物料分享格式预留兼容？预期：物料文件是单文件 `.rhai`，可附带 `<name>.meta.json` 存储作者/版本/预览图等元信息（本期不实现）。
