## ADDED Requirements

### Requirement: 物料创作指南文档

文档站 SHALL 提供面向用户的物料脚本创作指南（`docs/guide/material-scripting.md`），按五步创作流程组织：选图元 → 定布局 → 抽参数 → 声明 `defaults()`/`schema()` → 验证。文档 MUST 覆盖三函数约定、参数 widget 类型清单、动态输入 API、沙箱限制，且 MUST 包含至少一个完整可运行的物料示例。

#### Scenario: 用户按指南完成首个物料

- **WHEN** 用户依照 `material-scripting.md` 编写一份包含 `build` / `defaults` / `schema` 三函数的 `.rhai` 文件并放入用户物料目录
- **THEN** 该物料 MUST 能被 `Material::load` 成功加载
- **AND** 前端物料列表 MUST 显示该物料且可添加到图层

#### Scenario: 指南中的示例代码可直接运行

- **WHEN** 提取文档中的完整示例物料代码保存为 `.rhai` 文件
- **THEN** `Material::load` SHALL 加载成功
- **AND** 调用 `evaluate` 使用默认参数 MUST 返回非空 Element 列表

### Requirement: 图层使用文档

文档站 SHALL 提供图层管理使用说明（`docs/guide/layers.md`），覆盖图层层概念、叠加顺序（数组序 = 渲染序）、变换 / 样式 / 可见性，以及图层与 Profile 的关系。两篇新文档 MUST 在 VitePress 侧边栏注册且文档站构建无死链。

#### Scenario: 文档站侧边栏可导航到新文档

- **WHEN** 构建并打开文档站
- **THEN** 侧边栏 MUST 出现"物料脚本创作"与"图层管理"条目
- **AND** `npm run docs:build` SHALL 构建成功且无死链警告

### Requirement: 示例物料库

仓库 SHALL 在 `crates/material/examples/` 提供至少 3 个示例物料脚本，覆盖静态、时间动态、输入动态三类。每个示例 MUST 有顶部中文注释说明用途与参数，且 MUST 通过单元测试验证可被 `Material::load` 加载并成功求值。

#### Scenario: 示例物料通过加载与求值测试

- **WHEN** 运行 `cargo test -p peregrine_material`
- **THEN** 每个 `examples/*.rhai` 示例 MUST 被 `Material::load` 成功加载
- **AND** 以默认参数求值 MUST 返回非空 Element 列表

### Requirement: 内置物料目录只含正式内置物料

`crates/material/builtin/` MUST 只包含对应旧样式或明确设计为内置的物料。误置的示例性物料（如动态时钟 `time.rhai`）SHALL 移至 `examples/`；若既有配置已引用其 id，则 MUST 保留为内置并在创作指南中说明其定位。

#### Scenario: time.rhai 归位后无悬空引用

- **WHEN** `time.rhai` 从 `builtin/` 移出
- **THEN** 默认配置与迁移逻辑 MUST NOT 引用 `builtin.time`
- **AND** `cargo test -p peregrine_material` 全部通过

### Requirement: README 描述与四层架构同步

`README.md` 与 `README.zh-cn.md` 中的样式清单描述 MUST 更新为四层架构（元素 / 物料 / 图层 / 配置 + 用户可编程物料），不再停留在旧的固定样式枚举描述。

#### Scenario: README 不再描述封闭样式集

- **WHEN** 阅读 README 的特性列表
- **THEN** MUST 提及用户可通过 Rhai 物料脚本自定义样式
- **AND** MUST NOT 仅以旧 `CrosshairStyle` 枚举清单作为样式能力描述
