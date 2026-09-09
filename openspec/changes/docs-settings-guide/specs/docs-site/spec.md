## ADDED Requirements

### Requirement: 设置详解导航入口

文档站侧边栏 SHALL 包含「设置详解」（settings）导航项（双语），位于指南目录内；若其加入 Starlight 分页序列导致顺序异常，页面 frontmatter MUST 显式修正 `prev` / `next`（参照 `usage.md` 的既有模式）。

#### Scenario: 读者从侧边栏进入设置详解

- **WHEN** 读者在文档站任意指南页点击侧边栏「设置详解」
- **THEN** 浏览器 MUST 打开 `guide/settings`（en）或 `zh-cn/guide/settings`（zh）页面且分页上下文正确（上一页 / 下一页不指向 Download 页）

### Requirement: 配置说明页结构升级

`guide/config.md`（双语）SHALL 以新格式（`settings` 全局块 + `layers` 图层列表）作为正文示例与主要说明对象，并提供 `AppSettings`（含 `MaterialSettings`）逐字段表；旧 `crosshair` 单格式内容 SHALL 保留在明确标注的「遗留格式」章节中并附迁移语义说明（旧文件加载时自动迁移为 layers，保存后 `crosshair` 字段消失）。

#### Scenario: 旧格式章节可被老用户定位

- **WHEN** 持有旧版配置文件的用户在 `config.md` 中搜索 `crosshair` 字段
- **THEN** 页面 MUST 在「遗留格式」章节给出该字段说明，并提示该格式会被自动迁移、无需手工转换

#### Scenario: 字段表与 schema 同步声明

- **WHEN** 读者查阅 `AppSettings` / `MaterialSettings` 字段表
- **THEN** 表区附近 MUST 注明字段以 `crates/config/src/schema.rs` 为唯一事实源，文档冲突时代码优先
