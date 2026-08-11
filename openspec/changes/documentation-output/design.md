# Design：通用文档优化产出

## 背景

本 change 收敛两块「代码已稳定、文档被有意延后」的欠账：
1. 物料文档域（源自 `material-docs-examples` / `four-layer-customization` §19）
2. 遥测文档域（源自 `add-glitchtip-telemetry` §8/§9.3/§9.4，代码与 Windows 实机验收已完成）

两域同质（纯文档 / 示例交付、不改变运行时行为、依赖统一「照抄→修改→验证」流程），故合并为一个文档专项 change。

## 决策

### 决策 1：合并而非各自交付
**选择**：两块文档欠账合并到单一 `documentation-output` change。
**理由**：各自交付会让 `add-glitchtip-telemetry` 继续等待文档、`material-docs-examples` 独立交付后仍遗留遥测文档。合并后一次性产出，减少 change 上下文切换成本，且文档站侧边栏 / README / `AGENTS.md` 等公共文件改动只发生一次。
**替代方案（否决）**：保留两个独立 change —— 增加协调成本，无收益。

### 决策 2：capability 划分按功能域
**选择**：两个独立 capability `material-docs` 与 `telemetry-docs`，各自 spec 互不耦合。
**理由**：归档后 main specs 仍按功能域可检索（遥测文档归属遥测能力簇、物料文档归属物料能力簇），避免将来「文档」这类模糊 capability。
**原 `material-authoring-guide` 目录改名**：为反映合并后范围，`specs/material-authoring-guide/` 重命名为 `specs/material-docs/`（spec 内容不变）。

### 决策 3：REPORT_CODES.md 放仓库根而非 docs/
**选择**：`REPORT_CODES.md` 置于仓库根目录（与 `AGENTS.md` / `CHANGELOG.md` 同级），不进 VitePress 文档站。
**理由**：该文档面向贡献者 / AI 代理登记上报 Code，属开发者向治理文档，不是终端用户阅读内容。放仓库根便于 AI 代理与 PR review 第一时间发现；文档站开发指南（`development.md`）会链接到它。
**替代方案（否决）**：放 `docs/guide/` —— 会与用户向内容混杂，且 `docs/` 部署到 GitHub Pages 后治理文档被公开搜索，无必要。

### 决策 4：隐私说明独立成页
**选择**：新建 `docs/guide/privacy.md`（中英镜像），而非塞进现有 `features.md` 或 `config.md`。
**理由**：隐私是用户高频关注、且适合被外部（应用商店 / 论坛）直接链接的独立主题；独立成页便于维护与检索。设置页首次启动授权弹窗与 README 也会链接到它。

### 决策 5：time.rhai 归位延后到实现期判定
**选择**：不在 design 阶段定死 `time.rhai` 去留，实现时先 grep 默认配置 / 迁移逻辑是否引用 `builtin.time`，无引用则移至 `examples/`，有引用则保留。
**理由**：判定依赖代码现状（默认配置 + 迁移逻辑），design 阶段写死易过时。spec 已用「若…则…」条件式表达，实现期据实决策即可。

## 数据流

```
原 change 任务迁移：
  material-docs-examples (全部)        ─┐
  add-glitchtip-telemetry §8/§9.3/§9.4 ─┼─→ documentation-output
                                         │
交付物：                                  │
  docs/guide/material-scripting.md ──────┤  (物料文档域 → material-docs spec)
  docs/guide/layers.md ──────────────────┤
  crates/material/examples/*.rhai ───────┤
  README + AGENTS 物料段 ────────────────┘
  REPORT_CODES.md ───────────────────────┐
  docs/guide/privacy.md ─────────────────┤  (遥测文档域 → telemetry-docs spec)
  docs/guide/development.md 遥测章节 ────┤
  AGENTS.md 遥测章节 ────────────────────┘
  docs/.vitepress/config.mts (公共：侧边栏注册)
```

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| 文档站构建因新文档引入死链 | tasks §验证强制 `cd docs && npm run docs:build` 无死链警告 |
| `REPORT_CODES.md` 与常量模块漂移 | spec 场景要求「文档与代码对齐」核验；归入 CI 非强制，靠 review 把关 |
| `time.rhai` 移出导致既有配置断引用 | 实现前 grep 核查，有引用则保留（spec 条件式表达） |
| 物料示例脚本动态 API（time_ms 等）在动态输入软关闭下不可实测 | 示例只验证 `Material::load` 加载 + 静态参数求值成功，不验证动态运行时效果（与软关闭状态一致）|

## 实现顺序建议

1. 先做遥测文档域（`REPORT_CODES.md` + `privacy.md` + `development.md` 章节 + `AGENTS.md` 章节）——内容来源已稳定（代码已验收），可立即产出，立即解除 `add-glitchtip-telemetry` 的文档阻塞。
2. 再做物料文档域（创作指南 + 图层说明 + 示例物料 + time.rhai 归位 + README）。
3. 最后统一注册侧边栏、构建文档站验证无死链。
