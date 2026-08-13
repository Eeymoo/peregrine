# 变更提案：allow-empty-profile-layers

> **状态**：draft
>
> **跟踪 issue**：#45（https://github.com/Eeymoo/peregrine/issues/45）

## Why

GlitchTip 线上事件（PGR-3003，issue #13）暴露：多图层模式下用户删除最后一个图层时，`Profile::validate` 的「profile has neither layers nor crosshair」校验拒绝持久化，前端 toast 后 Promise 继续 reject 且无人 catch，形成 unhandled rejection 噪音上报。更根本的问题是**前后端对「0 图层是否合法」的约定不一致**：前端 `LayerPanel` 已有空图层列表的空态 UI（`layers.empty`），设计上预见了该状态；后端却将其视为配置损坏。同时发现渲染暗礁：空 layers 时预览渲染空白，但 overlay 会掉入旧 crosshair 路径回退画出一个用户从未配置的默认准星，违反 WYSIWYG 原则。

## 目标

- 放行空 layers：`layers 为空 && crosshair 为 None` 成为合法配置状态（语义：「当前不显示任何锚点」）
- 建立渲染不变量：**overlay 运行中 ⟹ 活跃 profile 至少有 1 个可见图层（或 legacy crosshair）**
- 对偶约束：渲染中禁止删除/隐藏「最后一个可见图层」；未渲染时允许删空/全隐藏，但无可渲染内容时禁止启动覆盖
- 修复渲染暗礁：0 可见图层时 overlay 渲染空白（与预览一致），不再回退默认准星
- 消除 PGR-3003 遥测噪音：`LayerPanel` 各 handler 补 `.catch(() => {})`

## 非目标

- **不拦截 overlay 运行中切换 profile 到无可见层 profile**：结果为渲染空白，可恢复，显式不处理
- **不拦截 watcher 外部编辑配置文件清空图层**：同上，渲染空白即可
- **不改动 `update_layer` 其他字段的校验**：仅对 `visible: false` 补丁新增最后可见层保护
- **不改动 legacy crosshair 路径的行为**：纯旧配置（`crosshair=Some` 且 layers 空）照常启动与渲染
- **不新增遥测 report code**：复用现有 PGR-3003 通道，噪音消除后该通道自然静默

## What Changes

- **schema 放宽**：`Profile::validate` 删除「neither layers nor crosshair」Err 分支（`crates/config/src/schema.rs`），空 layers + None crosshair 合法；配套更新 schema 单元测试（错误用例 → 合法用例）
- **渲染判定修正**：`overlay_renderer.rs` 的 `use_new_format` 判定从「layers 非空」改为「`crosshair.is_none()` 或 layers 非空」——crosshair 的存在与否才是真正的新旧格式标志；0 可见图层时渲染空白
- **`start_overlay` 硬校验**：活跃 profile 无可渲染内容（可见图层数 == 0 且 crosshair 为 None）时返回 `Err`（走 translate 双语文案）
- **`remove_layer` 硬校验**：overlay 活动且被删图层是最后可见层且 crosshair 为 None 时返回 `Err`
- **`update_layer` 硬校验**：`patch.visible == Some(false)` 且该层是最后可见层且 overlay 活动时返回 `Err`
- **统一判定谓词**：新增「该操作后活跃 profile 是否将无可渲染内容」辅助函数，三处硬校验复用
- **前端 UX 层**：开始覆盖按钮在可见图层为 0 时禁用 + tooltip；`LayerPanel` 传入 `overlayActive`，渲染中禁用最后可见层的删除/隐藏按钮 + tooltip；各图层操作 handler 补 `.catch(() => {})`（invoke 包装已负责 toast）
- **i18n**：新增 2~3 条文案，6 语 locale 同步

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `layer-composition`：Profile 校验语义变化（空 layers 合法化）；`remove_layer` / `update_layer` 新增渲染中保护约束；新增「渲染不变量」需求；`start_overlay` 新增可渲染内容前置校验

## Impact

- **代码**：`crates/config/src/schema.rs`（validate + 测试）、`crates/peregrine/src/overlay_renderer.rs`（use_new_format 判定）、`src-tauri/src/lib.rs`（start_overlay / remove_layer / update_layer 校验 + 辅助谓词）、`src/components/LayerPanel.tsx`（禁用逻辑 + catch）、`src/ConfigApp.tsx` 或 `src/components/LayersEditor.tsx`（overlayActive 透传、开始覆盖按钮禁用）、`src/i18n/locales/*.json`（6 语文案）
- **行为兼容**：此前会被 `load_or_create_default` 当作损坏备份回退的「空 layers + None crosshair」磁盘配置，放行后合法保留——属期望效果（用户主动清空不再被误伤）
- **遥测**：PGR-3003（UNHANDLED_REJECTION）针对该场景的噪音消除
- **依赖**：无新增依赖
