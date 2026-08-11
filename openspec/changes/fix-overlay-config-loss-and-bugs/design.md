## Context

v0.2.1 稳定版引入了四层架构（Elements / Materials / Layers / Config），多图层编辑链路是这次重构的核心成果。但演示与实测暴露出一连串问题，根因集中在两条链路：

1. **前后端 config 同步协议不一致**：单图层路径用「全量 saveConfig」，多图层路径用「字段级 updateLayer」，但 `LayersEditor.updateTargetWindow` 错误地复用了单图层的全量路径——用前端内存态 config 覆盖后端，导致后发的图层变更全部丢失。
2. **错误处理在多图层链路整体缺席**：后端返回 `Result<_, String>` 导致前端 Promise reject 出来的是字符串而非 Error 对象，Sentry 报「Non-Error promise rejection」；前端图层操作全部没有 try/catch，用户看不到任何错误反馈。

其它问题（AutoSwitchDialog 不渲染、grid 算错、dead parameter、透明度显示、slider 上限）都是相对独立的局部 bug，但一并修才让多图层真正可用。

### 当前架构（修复前）

```
配置同步协议（多图层路径）
═══════════════════════════

前端 ConfigApp state          后端 shared snapshot
─────────────────────         ──────────────────────
config.profiles.X.layers      config.profiles.X.layers
   ↑ 初始化 getConfig()          ↑ 权威源
   │ 不被 update_layer 同步       │
   │                             │
   │                             │ ← addLayer / updateLayer
   │                             │   字段级 patch（正确）
   │                             │
   │  saveConfig(newConfig)      │
   └─────────────────────────────┴── 全量覆盖（错误！）
       ↑ 用旧 config.profiles.X.layers
         覆盖后端最新值
```

```
错误反馈链路（修复前）
══════════════════════

后端 Result<T, String>
   ↓ Tauri IPC
前端 Promise.reject("字符串")  ← 不是 Error 对象
   ↓
.catch(console.error)         ← 静默吞掉
   ↓
⚠️ Sentry: Non-Error promise rejection
用户：什么都看不到
```

## Goals / Non-Goals

**Goals:**

- **消除任何全量覆盖后端的路径**：所有 profile 字段变更走 patch API，layers 字段永远只在后端维护
- **错误可见**：用户改参数失败时立即看到 toast/inline 提示，错误对象是标准 Error
- **全局对话框与编辑模式解耦**：AutoSwitchDialog 等组件挂载点不依赖 layersMode
- **物料脚本行为正确**：grid 的 grid_size 真实生效、dead parameter 有明确处理（实现或移除）
- **UI 显示符合常识**：透明度 0-1 显示为 0-100%、slider 上限覆盖 4K 屏
- **单一 PR 落地**：错误处理改造是 BREAKING，不能拆 PR 分批 merge

**Non-Goals:**

- 不做 slider max 自适应屏幕宽度（复杂度中高，单独 issue）
- 不重构物料求值缓存（每次 evaluate 都跑 Rhai，虽然注释说有缓存但实际没有）
- 不实现 random_orb.mode 的 lock_on_start 真实行为（dead parameter 按决策处理）
- 不扩展错误码体系（不新增 PGR-XXXX telemetry report code）
- 不重构单图层模式的 `saveConfig`（保留兼容，仅消除多图层路径下的全量调用）

## Decisions

### D1: 新增 `update_profile_field` patch API（根治配置丢失）

**决策**：新增后端命令 `update_profile_field(profile_name: String, field: ProfileField, value: serde_json::Value)`，按字段名 patch 更新 profile 顶层字段（target_window / settings_hotkey），**不触及 layers 字段**。

**备选方案**：

| 方案 | 描述 | 优点 | 缺点 | 结论 |
|---|---|---|---|---|
| A. 新增 patch API | `update_profile_field(name, field, value)` | 根治、字段级 | 需要新命令 | ✅ **采用** |
| B. layers-changed 同步整个 config | 监听事件时 getConfig → setConfig | 简单 | 仍有 race（事件到达前用户点保存） | 作为**兜底**配合 A |
| C. saveConfig 前先 getConfig | saveConfig 内部 await getConfig 再 merge | 安全 | 增加 IPC 往返、语义混乱 | ❌ 不采用 |
| D. save_config 后端做 merge | 比对差异保留字段 | 治本 | 难定义语义、改动大 | ❌ 不采用 |

**A+B 组合**：A 治本（不再全量覆盖），B 兜底（万一未来有别的路径触发类似问题，前端 config 也能自动同步）。

**ProfileField 枚举设计**：

```rust
#[serde(tag = "kind", rename_all = "snake_case")]
enum ProfileFieldUpdate {
    TargetWindow { value: String },
    SettingsHotkey { value: String },
    // 未来可扩展：Trigger 等
}
```

避免传 `serde_json::Value` 后端反序列化麻烦，用强类型 enum。

### D2: IPC 错误协议改造为结构化对象

**决策**：所有图层/profile 命令的错误返回类型从 `Result<T, String>` 改为 `Result<T, IpcError>`，其中：

```rust
#[derive(Debug, serde::Serialize)]
struct IpcError {
    code: String,       // 稳定错误码，如 "VALIDATION" / "NOT_FOUND" / "INTERNAL"
    message: String,    // 人类可读的错误信息（中文）
}
```

Tauri 序列化时 reject 出来的是对象，前端 `reject({code, message})`——但仍不是 Error 对象。

**前端包装**（`api.ts`）：

```typescript
async function invoke<T>(cmd: string, args?: object): Promise<T> {
  try {
    return await tauriInvoke<T>(cmd, args);
  } catch (e) {
    // Tauri IPC reject 出来的是 IpcError 对象或字符串
    const msg = typeof e === "string" ? e : (e?.message ?? String(e));
    const code = typeof e === "object" && e !== null ? e.code : "UNKNOWN";
    const err = new Error(msg);
    (err as any).code = code;
    showToast(msg);  // 统一 toast 显示
    throw err;
  }
}
```

**备选方案**：

| 方案 | 描述 | 结论 |
|---|---|---|
| A. 结构化 IpcError + 前端包装 | `{code, message}` + `throw new Error` | ✅ **采用** |
| B. 后端直接返回 anyhow::Error | 自动 display 为字符串 | ❌ 退化为 String |
| C. 前端手动在每个 catch 点 new Error | 散落各处 | ❌ 样板代码 |

### D3: 全局对话框层移到 ConfigApp 顶层

**决策**：把 `<AutoSwitchDialog>` / `<UpdateDialog>` / `<UpdateProgress>` 三个组件从单图层 return 内移到 ConfigApp 最外层 `<ErrorBoundary>` 内、`layersMode return` 之前。

**结构变更**：

```tsx
// ConfigApp.tsx 新结构（伪代码）
if (loading || !config) return <Loading />;
if (!crosshair && !hasLayers) return <ErrorPage />;

return (
  <>
    {layersMode ? <LayersEditor ... /> : <SingleLayerEditor ... />}

    {/* 全局对话框层 —— 与编辑模式解耦 */}
    {showAutoSwitchDialog && <AutoSwitchDialog ... />}
    {updateAvailable && !updating && <UpdateDialog ... />}
    {updating && <UpdateProgress ... />}
  </>
);
```

把单图层/多图层视图统一成一个三元表达式，全局对话框作为兄弟节点挂载。

### D4: grid.rhai 算法修复

**决策**：cols/rows 改用 `floor` 取整，用 `cell * cols` 计算 `total_w`：

```rhai
let cols = to_int(w / cell);   // floor：能完整放下的格子数
if cols < 1 { cols = 1; }
let rows = to_int(h / cell);
if rows < 1 { rows = 1; }
```

edge 模式实际 cell_w 用 `cell`（用户设定值），不再用 `w / cols` 重算。center 模式 `total_w = cell * cols` 保证不超屏。

**影响**：已保存的 grid_size 值在新算法下渲染更准确，但视觉上格数可能变少（之前 ceil 多算一格）。属于 bug 修复，接受视觉差异。

### D5: dead parameter 处理决策

**决策表**：

| 参数 | 决策 | 理由 |
|---|---|---|
| `border_frame.inset` | **实现** | 贴边/跨边渲染逻辑简单（控制 offset 的符号或位置），实现成本低 |
| `edge_rect.corner_radius` | **实现** | 需要渲染器支持圆角矩形（Rect shape 增加 corner_radius 字段），改动稍大但价值明确 |
| `random_orb.center_deviation` | **实现** | 中心规避逻辑（生成位置后判断距中心距离，超出则重新生成），中等复杂度 |
| `random_orb.mode` | **隐藏** | AGENTS.md 已记录为待办，lock_on_start 需要持久化随机种子 + 跨帧一致，复杂度高；UI 加 disabled + "coming soon" 提示 |

**Element 协议扩展**（edge_rect.corner_radius 实现需要）：

```rust
// crates/config/src/schema.rs
Element::Rect { x, y, w, h, corner_radius: Option<f32> }
```

`Option<f32>` 保持向后兼容（旧物料不输出该字段，反序列化为 None）。

渲染器（`overlay_renderer.rs` 的 CPU 光栅化 + svg_renderer）需要支持圆角矩形——CPU 路径可用 `paths` 或简单的四角圆弧填充；SVG 路径直接输出 `<rect rx="...">`。

### D6: SliderField 增加 format 回调

**决策**：SliderField 新增 `format?: (v: number) => string` 可选参数，默认不传时保持现有行为（直接展示 value + unit）。

```tsx
interface SliderFieldProps {
  // ... 现有字段
  format?: (v: number) => string;
}

// 渲染
<input value={format ? format(value) : value} ... />
{unit && !format && <span>{unit}</span>}
```

**透明度场景**：

```tsx
<SliderField
  value={style.opacity}
  min={0} max={1} step={0.01}
  format={(v) => Math.round(v * 100) + "%"}
  ...
/>
```

**备选方案**：

| 方案 | 描述 | 结论 |
|---|---|---|
| A. format 回调 | 通用、可复用（角度、字节等） | ✅ **采用** |
| B. 专款专用 displayValue + multiply | 只解决透明度 | ❌ 过度专用 |
| C. 在每个调用点条件渲染 | 散落 | ❌ 样板代码 |

### D7: 物料 slider max 扩充策略

**决策**：按分级表统一调整所有内置物料 schema 的 max，不改 min/step/default。分级标准：

| 参数语义 | 新 max | 例子 |
|---|---|---|
| 距离/偏移/尺寸/位置 | 1920 | offset / margin / distance / tail / size / grid_size / offset_x/y |
| 半径 | 500 | radius / radius_min / radius_max |
| 线粗 | 50 | thickness |
| 间隙 | 200 | gap |
| 缩放倍数 | 50 | scale |
| 字体大小 | 400 | font_size |
| 数量/比例 | 不变 | count / orb_count / *_pct |

**理由**：1920 是常见屏幕宽度（FHD），覆盖大多数场景；4K (3840) 留待自适应方案解决。

### D8: 错误 toast 复用 globalErrorToast 基础设施

**决策**：`api.ts` 的统一 invoke 包装复用 `globalErrorToast.ts` 的 `showToast` 函数（已存在于 `src/lib/globalErrorToast.ts`），避免重复造轮子。

`showToast` 当前签名：`showToast(message: string, stack?: string)`，正好适配 IPC 错误的 message 字段。

## Risks / Trade-offs

| 风险 | 影响 | 缓解 |
|---|---|---|
| **IPC 协议 BREAKING** | 前后端必须同步落地，不能拆 PR | 单一 PR 全量改造；改造前 `cargo test` + `npm run build` 双验证；手测覆盖所有图层操作 |
| **grid 算法变更影响现有用户** | 已保存的 grid_size 渲染结果变化 | 属于 bug 修复（之前是错的），在 release notes 说明；不迁移旧值 |
| **edge_rect 圆角渲染** | CPU 光栅化圆角矩形实现复杂 | 优先实现 SVG 路径（`<rect rx>`）；CPU 路径降级为直角，warn 日志 |
| **dead parameter 全部实现工作量** | random_orb.center_deviation 实现需要改动 build 逻辑 | 按 D5 决策表分优先级，先简后难；mode 隐藏而非实现 |
| **update_profile_field 强类型 enum** | 添加新字段需要改 Rust 枚举 | 可接受，profile 顶层字段稳定，不会频繁扩展 |
| **format 回调滥用** | 调用方可能传奇怪 format 函数 | 文档约束「仅用于显示格式化」；类型签名清晰 |
| **layers-changed 同步整个 config 的性能** | 每次 layers 变化都触发 getConfig + setConfig | getConfig 是内存读取（无 IO），React 状态更新成本低，可接受 |

## Migration Plan

本次变更是**单次集中修复**，不分阶段：

1. **后端先行**：新增 `update_profile_field` 命令 + IPC 错误类型改造（不改前端，前端兼容旧 String 错误）
2. **前端跟进**：`api.ts` 统一包装 + 图层操作加 try/catch + LayersEditor.updateTargetWindow 改走 patch API
3. **物料脚本**：grid.rhai 修复 + dead parameter 按决策实现/隐藏
4. **UI 显示**：透明度 format + 全局对话框层
5. **slider max**：批量调整
6. **README 视频**：独立小改动

**回滚策略**：单一 PR 整体 revert 即可（不涉及数据迁移、不涉及配置文件格式变更）。

## Open Questions

- **random_orb.center_deviation 实现细节**：中心规避算法用「拒绝采样」（生成后判断距中心，超出阈值则重试 N 次）还是「极坐标偏移」（在允许半径外生成）？倾向拒绝采样（简单）。
- **edge_rect 圆角 CPU 渲染**：是否接受降级为直角（仅 SVG 后端支持）？倾向接受，warn 日志提示。
- **update_profile_field 是否合并 updatePreferences**：两者都改 profile.settings_hotkey 之外的字段？不，updatePreferences 改 AppSettings（全局），update_profile_field 改 Profile 内字段，语义不同，保持分离。
