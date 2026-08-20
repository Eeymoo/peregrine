## 1. 截图管线扩展

- [x] 1.1 探测截图环境：启动 root Vite dev server（:5199），重跑现有 `npm run screenshots` 验证 `settings-layers.png` 可再产出（Chromium 可用性确认）
- [x] 1.2 扩展 `docs/scripts/capture-screenshots.mjs`：按 Tab 依次点击（通用 / 覆盖层 / 物料 / 快捷键），每 Tab 等待渲染稳定后截图，产出 `settings-general.png` / `settings-overlay.png` / `settings-material.png` / `settings-hotkeys.png` 入既有截图输出目录
- [x] 1.3 复跑 `npm run screenshots` 验证可重复执行且覆盖旧图；若环境不可行，在 PR 中注明阻塞原因并保持本组任务未勾选状态说明

## 2. config.md 翻新（双语）

- [x] 2.1 将正文 JSON 示例升级为新格式（`settings` 全局块 + `layers`，与 `Profile::default_profile()` 一致）
- [x] 2.2 新增 `AppSettings` 逐字段表（15 字段，对照 `crates/config/src/schema.rs` doc 注释与默认值函数），含 `MaterialSettings` 子表；表区注明以 schema.rs 为唯一事实源
- [x] 2.3 将 `Crosshair` 大字段表与枚举说明移入「遗留格式（Legacy）」章节，前置自动迁移说明（旧文件加载即迁移为 layers，保存后 `crosshair` 字段消失）
- [x] 2.4 同步产出 en 版 `config.md` 镜像页

## 3. settings.md 新增（双语）

- [x] 3.1 撰写 zh 版 `guide/settings.md`：总览（Tab 索引图）+ 通用 / 覆盖层 / 物料 / 快捷键逐项说明（作用 / 默认值 / 生效方式：即时 vs 需重启）+ 更新细微介绍；不写关于与开发者
- [x] 3.2 物料章节配两层与门 ASCII 示意图 + FPS 节拍语义表（system = 跟随主屏刷新率，回退 60；30/60/120 固定 cap；纯静态 profile 不受影响）
- [x] 3.3 覆盖层章节配渲染后端 CPU vs SVG 对比表
- [x] 3.4 快捷键章节配动作 ↔ 键位表与录制交互说明
- [x] 3.5 各章节嵌入对应真实截图（任务 1 产出的 4 张 PNG；若 1 阻塞则留占位并注明）
- [x] 3.6 同步产出 en 版 `guide/settings.md` 镜像页，逐节对照 zh 版
- [x] 3.7 侧边栏分页检查：如 settings 页分页指向异常，用 frontmatter `prev/next` 显式修正（参照 `usage.md` 模式）（构建产物验证：prev=Configuration、next=Material Scripting，无需修正）

## 4. 交叉链接与验收

- [x] 4.1 在 `usage.md` / `layers.md` / `material-scripting.md` / `getting-started.md`（双语）适当位置补链到 settings 页与翻新后的 config 页
- [x] 4.2 `cd docs && npm run build` 构建通过，双语页面均在构建产物中，图片路径无 404
- [x] 4.3 校对：字段表抽样与 `schema.rs` 逐字段对照（类型 / 默认值 / 可选枚举），双语内容逐节一致性检查
