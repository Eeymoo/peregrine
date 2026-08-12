# 设计文档：allow-empty-profile-layers

## Context

线上遥测（GlitchTip issue #13，PGR-3003）记录了完整事故链：用户在多图层模式下删除最后一个图层 → `remove_layer` → `persist_and_broadcast` 内 `config.validate()` 拒绝（`schema.rs:232`「profile has neither layers nor crosshair」）→ IPC reject → 前端 invoke 包装已 toast，但 `LayerPanel.handleDelete` 裸 `await` 无 catch → unhandled rejection 上报。

探索阶段确认的关键事实：

- **前后端约定不一致**：`LayerPanel.tsx:131` 已有 0 图层空态 UI（`layers.empty`），前端设计上允许该状态；后端校验禁止持久化该状态。
- **失败后状态侥幸一致**：`remove_layer` 只改配置 clone，校验失败提前返回，共享快照未污染；前端 `onChanged()` 因 throw 未执行，UI 与后端恰好一致——但这是侥幸而非设计。
- **渲染暗礁**：`overlay_renderer.rs:118` 的 `use_new_format = MATERIAL_RUNTIME_ENABLED && !layers.is_empty()`。空 layers 时掉入旧 crosshair 路径，`crosshair=None` 时经 `layer_to_crosshair(None)` 落到 `unwrap_or_else(Crosshair::default_crosshair)`——overlay 画出用户从未配置的默认准星。而预览路径 `build_shapes_ipc`（lib.rs:1617）在 `MATERIAL_RUNTIME_ENABLED=true` 时直接走 `build_layers_shapes`，空迭代返回空 vec（空白）。**预览与 overlay 行为分裂**，违反 WYSIWYG。
- 后端三处新校验（`start_overlay` / `remove_layer` / `update_layer`）语义相同，需要统一谓词避免三份漂移逻辑。
- `AppState.overlay_active: AtomicBool` 已存在（lib.rs:876 `get_overlay_active` 读取），后端做「渲染中」判定无新增状态。
- 前端 `overlayActive` 存在于 `useConfigAppState`，但 `LayerPanel` 目前拿不到，需要沿 `ConfigApp → LayersEditor → LayerPanel` 透传。

## Goals / Non-Goals

**Goals:**

- 空 layers + None crosshair 成为合法可持久化状态，语义为「当前不显示任何锚点」
- 渲染不变量：overlay 运行中 ⟹ 活跃 profile 至少有 1 个可见图层（或 legacy crosshair），由后端硬校验保证、前端禁用兜底
- 预览与 overlay 在 0 可见图层时行为一致（都渲染空白）
- 消除该场景的 PGR-3003 unhandled rejection 噪音

**Non-Goals:**

- 不拦截运行中切换 profile / watcher 外部编辑导致的空渲染（可恢复，渲染空白即可）
- 不改 legacy crosshair 路径既有行为
- 不重构 IPC 错误协议（沿用现有 `Result<T, String>` + invoke 包装 toast）
- 不新增遥测 report code

## Decisions

### 决策 1：「最后一个」按「最后可见图层」判定

约束对象是所有使**可见图层数归零**的操作，而非「图层总数归零」。否则存在漏洞：3 个图层全隐藏后 overlay 渲染空白，绕过不变量。

- 渲染中删除/隐藏判定：`profile.layers.iter().filter(|l| l.visible).count()`，操作后将为 0 且 `crosshair.is_none()` → 拒绝。
- 推论：渲染中删除**已隐藏**的图层恒放行；渲染中可见图层 ≥2 时删除/隐藏任一放行。
- `start_overlay` 同理：可见图层数 == 0 且 `crosshair.is_none()` → 拒绝（全部隐藏等价于空配置）。

**备选**：按图层总数判定——被否，隐藏按钮成为绕过通道。

### 决策 2：统一谓词收敛在 schema 层

新增 `Profile::has_renderable_content(&self) -> bool`（`crosshair.is_some() || layers.iter().any(|l| l.visible)`）。三处硬校验复用：

- `start_overlay`：`!profile.has_renderable_content()` → `Err(translate(...))`
- `remove_layer`：overlay 活动 && 删除后剩余 `visible` 层数 == 0 && `crosshair.is_none()` → `Err`
- `update_layer`：`patch.visible == Some(false)` && 该层当前可见 && 其余可见层数 == 0 && overlay 活动 && `crosshair.is_none()` → `Err`

放在 schema（`peregrine_config`）而非 `src-tauri`：纯数据判定，符合分层原则（config crate 不依赖 UI/平台），且可单测。overlay 活动状态由 `src-tauri` 侧读取 `overlay_active` 后组合，不进入 schema。

**备选**：三处各自内联判定——被否，条件细微差异（删除后 vs 当前）极易漂移。

### 决策 3：`use_new_format` 改以 `crosshair.is_none()` 为新格式标志

`overlay_renderer.rs:118` 判定改为：

```text
use_new_format = MATERIAL_RUNTIME_ENABLED
    && profile.map(|p| !p.layers.is_empty() || p.crosshair.is_none()).unwrap_or(false)
```

理由：迁移后的 profile `crosshair` 恒为 `None`（`migration.rs:112`），`crosshair` 的存在与否才是真正的「新旧格式」标志。空 layers + None crosshair 走新路径 → `build_layers_shapes` 空迭代 → 渲染空白，与预览一致。纯 legacy 配置（`crosshair=Some`）走旧路径，行为不变。

**备选**：在旧路径的 `unwrap_or_else(default_crosshair)` 前特判空 layers 返回空——被否，在错误的路径上打补丁，遗留「layers 空时新旧路径语义重叠」的结构性混乱。

### 决策 4：前端为 UX 层，后端 Err 为硬保证

- 前端：开始覆盖按钮在可见图层为 0 时 `disabled` + tooltip；`LayerPanel` 接收新 prop `overlayActive`，渲染中对「最后可见层」禁用删除/眼睛按钮 + tooltip。
- 后端：三处 Err 是最终防线（覆盖前端状态滞后、外部调用等场景）。
- `LayerPanel` 各 handler（`handleDelete` / `handleDuplicate` / `handleMove` / `handleToggleVisible` / `handleToggleLock`）统一补 `.catch(() => {})`：invoke 包装已 toast，catch 仅吞掉 rejection 防止上报。这与代码库既有惯例一致（`useConfigAppState.ts:60`、`useOverlayActions.ts:41`）。

### 决策 5：`load_or_create_default` 行为变化属期望效果

validate 放宽后，磁盘上「空 layers + None crosshair」的配置不再被当作损坏备份回退，而是合法保留。这正是目标语义——用户主动清空不应被误伤为配置损坏。其余校验（字段范围、枚举）不受影响。

## Risks / Trade-offs

- [validate 放宽削弱配置损坏防线] → 「空 layers + None crosshair」从损坏信号变为合法状态，是有意为之的产品决策；其他字段校验不变，损坏检测能力总体保留。
- [运行中切换 profile / watcher 外部编辑可绕过不变量，overlay 渲染空白] → 显式非目标；渲染空白可恢复（停止 overlay 或切回），且决策 3 保证此时 overlay 与预览一致地渲染空白而非幽灵准星。
- [`update_layer` 新增拒绝路径可能惊扰既有调用方] → 仅在「overlay 活动 && 最后可见层 && visible:false」三条件同时成立时拒绝；`MaterialParamControls` 不传 visible 补丁，不受影响。
- [前端 `overlayActive` 透传链路改动涉及 ConfigApp → LayersEditor → LayerPanel 三层] → 仅 prop 透传，无状态重构；快照不同步时以后端 Err 兜底。

## Migration Plan

无数据迁移：旧配置全部仍然合法（合法性集合只增不减）。部署后线上已存在的「空 layers」损坏配置（此前会被回退默认 + `.bak` 备份）将被保留原样——属期望行为。回滚策略：还原 validate 分支即可，无持久化格式变更。

## Open Questions

（无。语义边界「最后一个 = 最后可见」已在探索阶段与需求方确认。）
