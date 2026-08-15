# 设计：恢复动态物料端到端链路

## Context

`material-static-rendering` 将软关闭范围收窄为「仅动态输入」（`MATERIAL_DYNAMIC_INPUT_ENABLED = false`），静态多图层渲染活跃。但动态链路的调度机制在更早的 `disable-material-runtime` 中已被物理删除，且规格与代码存在多处漂移（无求值缓存却声称有、`now_ms()` 逃逸、`time.rhai` 移出示例）。本设计以最小机制恢复完整链路，不引入缓存、输入分级、秒对齐等优化。

关键现状（2026-08-14 审计确认）：

- `compute_is_animated`（overlay.rs:144）只认旧格式 RandomOrb。
- `frame_interval` 硬编码 16.67ms（overlay.rs:136）。
- `RefreshMaterials` 命令存在但退化为 `needs_redraw = true`（overlay.rs:202）。
- 物料 watcher 重建 registry 后既不发 overlay 命令也不 emit 事件（lib.rs:2355 TODO）。
- `examples/time.rhai` 用 `now_ms()`（墙钟直读），预览 IPC 与 overlay 求值时刻不一致。
- winit 0.30 `MonitorHandle::refresh_rate_millihertz()` 可提供主屏刷新率（毫赫兹 Option）。

## Goals / Non-Goals

**Goals:**

- 时钟物料（`builtin.time`）在 overlay 中按配置帧率持续跳动，实机可验证。
- 动态链路受「编译期总闸 + 运行时用户开关」双层控制，任一层关闭即冻结（安全回退路径完整）。
- 帧率档位 30/60/120/系统，`frame_interval` 随配置热更新。
- 预览中时钟实时跳动（前端 ~1s 定时器）。
- 预览与 overlay 的时间来源统一为 `DynamicContext`（消除 `now_ms` 逃逸）。
- 物料热重载后 overlay 无需重启即感知（`RefreshMaterials` 实义化 + watcher 接线）。

**Non-Goals:**

- 不做求值缓存（`DynamicContext.version` 继续无消费者，见 D5 记账）。
- 不做 `dynamic_inputs()` 输入声明协议 / 探针求值。
- 不做秒边界对齐调度（时钟按节拍器渲染，接受 ±1 帧相位抖动）。
- 不内置鼠标跟随 / 键盘响应物料（留在 `examples/`）。
- 不做性能基线与 1h 稳定性验证（宣告放弃，另立 change）。

## Decisions

### D1：编译期开关翻回 `true`，保留总闸语义

`MATERIAL_RUNTIME_ENABLED` 不动（已 `true`）；`MATERIAL_DYNAMIC_INPUT_ENABLED` 翻回 `true`（Rust + TS 成对）。编译期开关继续作为「整体回退软关闭」的一键闸门：两个常量翻回 `false` 即恢复 `material-static-rendering` 时代行为。

**备选**：彻底删除常量——否决，失去编译期回退闸门；且门控注释体系（grep 可枚举）已建立，维护成本低。

### D2：运行时用户开关 `settings.material.dynamic_enabled`（默认 true）

新增结构：

```rust
/// 物料运行时设置（「物料」Tab）。
#[serde(default)]
pub struct MaterialSettings {
    /// 动态物料总开关（运行时层）。默认 true。
    pub dynamic_enabled: bool,
    /// 动画帧率档位（FPS 上限节拍）。None = 跟随系统主屏刷新率（回退 60）。
    /// 仅接受 30 / 60 / 120。
    pub fps: Option<u32>,
}
```

生效语义为**双层与门**：动态链路活跃 ⇔ `MATERIAL_DYNAMIC_INPUT_ENABLED && settings.material.dynamic_enabled`。四个门控点（overlay 求值上下文 ×2、预览 IPC 上下文 ×1、LayerPanel 过滤 ×1）的判定统一改为该合取。

运行时开关关闭时：`compute_is_animated` 对 layers 恒 false（退化为现状）、选择器隐藏动态物料、求值走 `static_context()`——即用户侧的软关闭，重启不要求（`UpdateConfig` 热生效）。

**备选**：把运行时开关做成独立于编译期常量的第三态——否决，两层独立组合出的「编译期开 + 运行时关」语义与现状软关闭等价，无需第三态。

### D3：`frame_interval` 配置化 + 系统刷新率探测

- `OverlayApp::new` 不再硬编码 16.67ms：启动时从配置快照解析 FPS（`fps.unwrap_or(系统刷新率).unwrap_or(60)`），`UpdateConfig` 时重解析并热更新。
- 系统刷新率探测：winit `MonitorHandle::refresh_rate_millihertz()`（毫赫兹，四舍五入到 Hz），在 overlay 线程创建窗口后探测一次并缓存（跟随系统档不逐帧查询）；探测失败或异常值（<24 或 >480 Hz）回退 60。
- RandomOrb（旧格式动画）同样消费该节拍——行为从固定 60FPS 变为配置节拍，属预期变更（写入规格）。

**FPS 语义锁定**：`fps` 是「动画最高帧率节拍（cap）」而非「恒定渲染帧率」。纯静态 profile 保持 `ControlFlow::Wait` 纯事件驱动，不因 FPS 设置空转。

**备选**：`fps` 用 enum `System | Fps30 | Fps60 | Fps120`——否决，`Option<u32>` 序列化更简（缺失即系统），校验限枚举集即可；与 `telemetry_enabled: Option<bool>` 的「缺失 = 缺省」惯例同构。

### D4：`compute_is_animated` 直接计算，不恢复 `dynamic_dirty` 缓存

```rust
fn compute_is_animated(&self) -> bool {
    // 旧格式：RandomOrb（不随运行时开关门控，保持既有行为）
    // 新格式：任一可见图层物料 is_dynamic && 运行时开
}
```

判定开销：一次 config 锁 + 逐图层 HashMap 查找（registry 读锁）+ 布尔与，微秒级；现状 RandomOrb 判定即每轮直算（overlay.rs:512 注释），layers 遍历照抄该哲学。

**备选**：恢复 `dynamic_dirty` 缓存 + 失效点位（原 `overlay-dynamic-text-fixes` 决策 1）——否决，原设计自己承认「单次查询便宜」；缓存引入三个失效点位（UpdateConfig / RefreshMaterials / 创建）换不来可感知收益，且是当年规格与代码漂移的温床。`RefreshMaterials` 恢复「更新 registry 句柄 + needs_redraw」实义，但**不**携带 `dynamic_dirty`。

### D5：无求值缓存的性能取舍（记账）

`build_layers_shapes` 每帧全量求值（每图层新建 Rhai Engine + `call_fn`，单次 <100µs，e2e 基线曾测）。120FPS × 单动态图层 = 每秒 120 次求值 ≈ 12ms CPU（分散在 120 个 8.3ms 帧里，单帧占比 ~1.4%）。多动态图层线性放大。**本期接受**：目标是跑通；若实机验证发现可感知开销，后续 change 补 `version` 驱动的求值缓存（协议已在 `DynamicContext` 中预留）。

### D6：时钟物料按节拍器渲染，不做秒边界对齐

时钟在 30/60/120 档下均按节拍器渲染（每帧重求值，`time_ms()` 变化则文本变化）。不计算「下一秒边界 + 容差」的精准唤醒——那需要物料声明输入集（本 change 的 Non-Goal）。相位抖动 ≤1 帧节拍（120 档 8.3ms），视觉不可感。

### D7：时间 API 统一到 `DynamicContext`

- `builtin/time.rhai` 改用 `time_ms()`（上下文快照）：overlay 每帧求值拿到轮询时刻、预览 IPC 拿到请求时刻，两侧一致受调度节拍约束。
- `now_ms()`（墙钟直读）保留注册——既有用户脚本可能依赖，骤删是破坏性变更；文档标注「不推荐，新脚本用 `time_ms()`」。
- `examples/clock.rhai`、`examples/time.rhai` 同步改造为 `time_ms()`（示例的示范价值优先）。

**注**：`poll_dynamic_context` 的 `time_ms` 来自 `SystemTime`（墙钟），`context.rs::current_time_ms` 是「UNIX 起点 + Instant 单调」混合。两者数值都是墙钟毫秒，物料侧无差异；将来做缓存时统一到后者（单调性是缓存版本号的正确来源）。本期不改 `windows.rs`。

### D8：预览跳动用前端定时器，节拍独立于后端 FPS

`Preview.tsx`：检测 props 中 profile 含 `is_dynamic` 物料（经物料列表判定）且运行时开关开 → `setInterval(1000ms)` 重拉 `build_shapes_ipc`；否则清除定时器维持事件驱动。1s 是预览节拍（非后端 FPS）——预览无需 60/120Hz 刷新，秒级足以表达「这是活的」。组件卸载 / profile 切静态 / 开关关闭时清理定时器。

### D9：watcher 接线补 TODO

物料 watcher（lib.rs）重建 registry 后：

1. `app_handle.emit("peregrine:materials-changed", ())`（前端刷新物料列表用，原 TODO）；
2. 经 `EventLoopProxy` 向 overlay 线程发 `OverlayCommand::RefreshMaterials(new_registry)`（携带新句柄，overlay 侧替换 `material_registry` 字段）。

`RefreshMaterials` 变体签名从无载荷改为携带 `Arc<MaterialRegistry>`。热重载不改配置，故不触发 follower 重启逻辑，只置 `needs_redraw`（每轮直算的动态性判定下一轮自动反映新 `is_dynamic`）。

### D10：`time.rhai` 归位 `builtin/`

`examples/time.rhai` → `crates/material/builtin/time.rhai`，`BUILTIN_MATERIALS` 列表加入 `"time"`. 归位理由：用户开关的默认 true 需要**开箱即用的动态物料**来体现价值，否则「物料」Tab 的开关无实体可关。`examples/` 保留 `clock.rhai`（不同格式串示例）与另两份输入示例。

**迁移影响**：默认配置与迁移逻辑均不引用 `builtin.time`（`disable-material-runtime` 已清理），归位零迁移成本。引用 `builtin.time` 的存量图层（理论上不存在）直接生效。

## Risks / Trade-offs

- [120FPS 空转求值] → D5 记账接受；实机验证项包含 CPU 观察，超标另立 change。
- [两层开关组合状态多（2×2）] → 与门语义单一（全真才活），门控点统一合取表达式，规格以真值表锁定。
- [系统刷新率探测失败] → 60 兜底 + 异常值区间校验，规格锁定回退行为。
- [watcher 重建 registry 的竞态] → 沿用既有「整体替换 + RwLock」模式，`RefreshMaterials` 携带完整新句柄，无部分更新窗口。
- [`now_ms()` 保留导致的缓存击穿隐患] → 将来做缓存时以 `make_engine_with_dynamic` 的探针化收编（记录为 D5 后续路径），本期不处理。

## Migration Plan

1. 编译期常量翻转（一次 commit，可独立回退）。
2. schema + 设置 Tab + 门控改造（一次 commit）。
3. 调度 + watcher + time.rhai 归位（一次 commit）。
4. 预览跳动 + 前端同步（一次 commit）。
5. 规格同步 + AGENTS.md + 旧 change 归档（一次 commit）。

回滚：编译期常量翻回 `false` 即回到 `material-static-rendering` 行为（运行时开关与 FPS 设置变为无消费 UI 字段，无害）。

## Open Questions

（无——三个待决点已于 2026-08-14 探索会话锁定：运行时开关 / 本期做跳动预览 / 归档+取代说明。）
