## ADDED Requirements

### Requirement: 设置详解页面内容覆盖

文档站 SHALL 提供双语（en / zh-cn）的 `guide/settings.md`「设置详解」页面，逐项说明设置窗口的通用、覆盖层、物料、快捷键四个 Tab 的每一个设置项（含作用、默认值、生效方式），并对更新 Tab 做简要介绍（更新通道 stable / prerelease 与镜像加速）。

#### Scenario: 读者查找物料设置说明

- **WHEN** 读者打开「设置详解」页的物料章节
- **THEN** 页面 MUST 包含动态物料开关（`settings.material.dynamic_enabled`）与编译期总闸构成与门的说明、动画帧率（`settings.material.fps`：system / 30 / 60 / 120）的语义说明，并配有两层与门示意图

#### Scenario: 读者查找覆盖层渲染后端说明

- **WHEN** 读者打开「设置详解」页的覆盖层章节
- **THEN** 页面 MUST 包含渲染后端 CPU 与 SVG 的对比说明、抗锯齿开关与拖拽实时预览的行为说明

#### Scenario: 关于与开发者模式不出现在设置详解页

- **WHEN** 读者浏览「设置详解」页
- **THEN** 页面 MUST NOT 包含「关于」Tab 与「开发者模式」的专门章节

### Requirement: 设置详解页配真实截图

「设置详解」页的通用、覆盖层、物料、快捷键章节 SHALL 各配至少一张由截图管线产出的真实 UI 截图（`settings-general.png` / `settings-overlay.png` / `settings-material.png` / `settings-hotkeys.png`），双语页面共用同一组图片。

#### Scenario: 截图管线可重复产出各 Tab 截图

- **WHEN** 在 root Vite dev server（:5199）运行的状态下执行 `npm run screenshots`
- **THEN** 脚本 MUST 逐 Tab 点击切换并产出上述 4 张 PNG 到既有的截图输出目录，且重复执行结果覆盖旧图

### Requirement: 配置说明页与当前 schema 对齐

`guide/config.md`（双语）SHALL 与 `crates/config/src/schema.rs` 当前实现保持一致：正文 JSON 示例 MUST 为新格式（含 `settings` 全局块与 `layers` 图层列表），MUST 包含 `AppSettings` 与 `MaterialSettings` 的逐字段说明表，旧 `crosshair` 单格式 MUST 降级为「遗留格式」章节并附自动迁移说明。

#### Scenario: 读者按正文示例手写配置

- **WHEN** 读者按 `config.md` 正文 JSON 示例手写一份配置文件并启动应用
- **THEN** 该配置 MUST 被当前版本直接加载且不触发旧格式迁移路径（即正文示例本身就是新格式）

#### Scenario: 读者查阅全局设置字段

- **WHEN** 读者在 `config.md` 中查找 `settings.renderer_backend` 字段
- **THEN** 页面 MUST 在 `AppSettings` 字段表中给出其类型、默认值（`"cpu"`）与可选值（`"cpu"` / `"svg"`）说明
