# 通用文档优化产出

## Why

Peregrine v0.1.x 稳定线已落地多块新功能（多图层架构、物料运行时、GlitchTip 遥测集成），但文档侧欠账集中爆发：README 仍是旧的封闭样式清单、文档站缺少物料创作指南与图层使用说明、遥测功能已写完代码却无用户隐私说明与开发者上报 Code 登记、`AGENTS.md` 未同步遥测模块约定。这些欠账分属不同功能域但**同质**——都是「代码已稳定、只差文档一次性产出」，且都依赖「照抄 → 修改 → 验证」的统一工作流。本 change 将散落在多个原 change 中「有意延后」的文档任务（`material-docs-examples` 全部、`add-glitchtip-telemetry` 的 §8/§9.3/§9.4）统一收敛到一个文档专项，一次性交付，避免各功能 change 被文档阻塞归档。

## What Changes

### 新增

**物料文档域**（源自 `material-docs-examples`，迁移自 `four-layer-customization` tasks.md §19）：
- `docs/guide/material-scripting.md`：物料脚本创作指南——五步创作流程（选图元 → 定布局 → 抽参数 → 声明 `defaults()`/`schema()` → 验证），覆盖三函数约定、参数 widget 类型清单、动态输入 API（`time_ms` / `mouse_pos` / `key_down` / `rand`）、沙箱限制、完整可运行示例。
- `docs/guide/layers.md`：图层管理使用说明——图层层概念、叠加顺序（数组序 = 渲染序）、变换（offset / scale / rotation）、样式（颜色 / 不透明度 / 混合）、可见性与锁定、与 Profile 的关系、旧配置自动迁移后的单图层形态。
- `crates/material/examples/`：3 个示例物料脚本（静态、时间动态、输入动态），每个顶部带中文注释且能被 `Material::load` 加载并成功求值。

**遥测文档域**（源自 `add-glitchtip-telemetry` §8/§9.3/§9.4，代码与验收已稳定）：
- `REPORT_CODES.md`（仓库根）：开发者向的上报 Code 登记表——按号段（`PGR-0xxx` 启动 / `PGR-1xxx` panic / `PGR-2xxx` 后端 IO / `PGR-3xxx` 前端 / `PGR-4xxx` 遮盖层 / `PGR-5xxx` 操作域）登记所有已定义 Code，含语义、触发点、接线状态（已接 / 预留 / 豁免）。与 `telemetry.rs::report_code` / `src/lib/telemetry.ts::REPORT_CODES` 常量定义对齐。
- `docs/guide/privacy.md`（及 `docs/zh-cn/guide/privacy.md`）：用户向隐私说明——数据匿名保证（不采集 IP / 用户名 / 机器名 / 路径用户名）、上报三类事件（启动 / 崩溃 / 错误）的说明、开关位置与临时授权入口、`PEREGRINE_DISABLE_TELEMETRY` 编译期禁用选项的提示（面向自构建用户）。
- `docs/guide/development.md` 补充遥测开发章节：DSN 环境变量配置（`.env.development` / `.env.production` / `GLITCHTIP_DSN` / `GLITCHTIP_DSN_TEST`）、`PEREGRINE_DISABLE_TELEMETRY` 本地调试、`safe_try!` 使用约定（仅关键路径，0.2.0 接线 PGR-2101/4101）、Code 登记治理（新增上报点必须先在 `REPORT_CODES.md` 与常量模块登记）。
- `AGENTS.md` 新增「遥测模块」章节：模块定位（`src-tauri/src/telemetry.rs` + 前端 `src/lib/telemetry.ts`）、DSN 注入方式、隐私开关语义（`telemetry_enabled` 首次缺省 = 未授权）、`safe_try!` 约定、Code 治理约定、编译期禁用方式。

**公共**：
- `docs/.vitepress/config.mts`：注册 `material-scripting`、`layers`、`privacy` 三篇新文档的侧边栏条目（中英镜像）。
- `README.md` / `README.zh-cn.md`：旧样式清单更新为四层架构描述（元素 / 物料 / 图层 / 配置 + 用户可编程物料），并补一行隐私说明链接。

### 修改

- `crates/material/builtin/time.rhai` 归位评估：若无默认配置引用则移至 `examples/`，否则保留为内置并在创作指南中说明其动态物料范例定位。

### 删除

无。

## Capabilities

### New Capabilities

- `material-authoring-guide`：物料创作文档（创作指南 + 图层使用说明 + 示例物料库 + time.rhai 归位 + README 四层架构同步）
- `telemetry-docs`：遥测文档（REPORT_CODES 登记 + 用户隐私说明 + 开发文档遥测章节 + AGENTS.md 遥测章节）

### Modified Capabilities

（无——`openspec/specs/` 下无既有 capability 需要修改 delta）

## Impact

### 代码影响面

| 模块 | 影响等级 | 改动概要 |
|---|---|---|
| `docs/guide/` | 低 | 新增 `material-scripting.md` / `layers.md` / `privacy.md`（中英镜像） |
| `docs/guide/development.md` | 低 | 补遥测开发章节 |
| `docs/.vitepress/config.mts` | 低 | 侧边栏注册 3 篇新文档 |
| `crates/material/examples/` | 低 | 新增 3 个示例 `.rhai`（不进二进制，纯文件） |
| `crates/material/builtin/` | 低 | `time.rhai` 归位评估（若移出，`BUILTIN_MATERIALS` 同步调整） |
| `crates/material/` 测试 | 低 | 新增示例加载/求值单元测试 |
| `README.md` / `README.zh-cn.md` | 低 | 样式清单 → 四层架构 + 隐私链接 |
| `REPORT_CODES.md` | 低 | 新增开发者向 Code 登记表 |
| `AGENTS.md` | 低 | 新增遥测模块章节 |

### 依赖变更

无。

### 向后兼容

- 纯文档 / 示例交付，不改变任何运行时行为。
- 若 `time.rhai` 从内置移出：引用 `builtin.time` 的既有图层会失去物料（求值失败跳过该图层，不崩溃）。迁移策略：检查默认配置是否引用，若无引用则直接移出；若有引用则保留为内置。

### 来源标注

- 物料文档域：`four-layer-customization` tasks.md §19（19.2–19.6 假勾选修正），经 `material-docs-examples` change 起草。
- 遥测文档域：`add-glitchtip-telemetry` tasks.md §8 / §9.3 / §9.4，原标注「延后到代码 + 验收稳定后统一编写」，现遥测代码与 Windows 实机验收已完成，迁入本 change 产出。

### 发布版本

随下一 alpha（`v0.2.0-alpha.1`）或 stable（`v0.2.1`）发布，无独立版本要求。
