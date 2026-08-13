## ADDED Requirements

### Requirement: Tailwind v4 设计系统集成

文档站 MUST 通过 `@tailwindcss/vite` 与官方兼容包 `@astrojs/starlight-tailwind` 接入 Tailwind CSS v4：仅引入 theme 与 utilities 层（不含完整 preflight），暗色变体 MUST 跟随 Starlight 的主题开关而非系统偏好，且不得破坏 Starlight 内建排版。

#### Scenario: 构建不含 preflight 破坏

- **WHEN** 执行 `npm run build` 并打开任一 guide 页面
- **THEN** Starlight 的标题、列表、表格、code block 排版与未接入 Tailwind 前一致，无被 reset 的迹象

#### Scenario: 暗色跟随主题开关

- **WHEN** 用户在系统浅色偏好下手动将站点切到深色主题
- **THEN** 使用 `dark:` 工具类的 landing 组件同步切换为深色样式

#### Scenario: 官方集成路径

- **WHEN** 检查入口 CSS 与依赖
- **THEN** 使用 `@astrojs/starlight-tailwind` 兼容包与 `@layer base, starlight, theme, components, utilities` 声明，无手写的 preflight 或自造 dark 变体桥接

### Requirement: 品牌设计 token 单一事实源

品牌蓝与中性灰色阶 MUST 在 Tailwind `@theme` 中以 `--color-accent-*` 与 `--color-gray-*` 单处定义（Starlight 原生消费这两组 token），不得在其他 CSS 文件中手写相同色值。

#### Scenario: 色值单源

- **WHEN** 检查 `src/styles/` 下品牌蓝与中性灰色值的定义位置
- **THEN** 十六进制色值仅出现在 `@theme` 块内，其余文件通过 `var()` 或工具类引用

### Requirement: 首页落地页构成

首页（en 与 zh-cn）MUST 呈现现代化落地页结构：自定义 Hero（左文右图，含真实产品截图）、特性网格、三步上手区块、下载入口区块；且 MUST 保持 `template: splash` 与 Starlight 顶栏（语言切换、主题开关、GitHub 链接）可用。

#### Scenario: Hero 结构

- **WHEN** 打开 `/` 或 `/zh-cn/`
- **THEN** Hero 左侧为标题、不超过 20 词的副标题与至多两个 CTA，右侧为真实产品截图，首屏内 CTA 可见无需滚动

#### Scenario: 落地页区块完整

- **WHEN** 滚动首页
- **THEN** 依次出现特性网格、三步上手、下载入口三个区块，且各区块布局模式不重复

#### Scenario: 双语一致

- **WHEN** 对比 `/` 与 `/zh-cn/` 首页
- **THEN** 两者区块结构、组件构成完全一致，仅文案语言不同

#### Scenario: Starlight 顶栏保留

- **WHEN** 在首页点击语言切换或主题开关
- **THEN** 正常跳转对应 locale 或切换明暗主题，功能不受 Hero 覆写影响

### Requirement: 真实截图资产

首页使用的产品截图 MUST 来自真实渲染：React 设置面板 UI 在浏览器中加载，配合 Rust 侧 `build_layers_shapes` 导出的真实图元数据，经 headless Chromium 截取；不得使用 div 手绘的假界面或第三方图库占位图。截图管线 MUST 以脚本形式固化于 `docs/scripts/`，可重复执行。

#### Scenario: 多图层截图生成

- **WHEN** 执行截图脚本
- **THEN** 在 `docs/public/img/screenshots/` 生成展示多图层编辑态的设置面板截图，画面中的预览图元与 Rust 导出数据一致

#### Scenario: Hero 引用真实截图

- **WHEN** 检查首页 Hero 的图源
- **THEN** 引用 `public/img/screenshots/` 下的本地资产，无外链占位图

### Requirement: 文档页排版精修

guide 页面 MUST 在不覆写 Starlight 组件的前提下（仅通过 `customCss`），完成配色层次与排版细节校准，覆盖正文、code block、aside、表格、链接，且明暗双主题各自完整。

#### Scenario: 精修不覆写组件

- **WHEN** 检查 `astro.config.mjs` 的 `components` 配置
- **THEN** 除 `Hero` 外无其他 Starlight 组件覆写

#### Scenario: 双主题观感

- **WHEN** 在明暗两种主题下打开任一 guide 页面
- **THEN** 两种主题下正文对比度、code block、aside、表格样式均完整可读，无未适配的裸默认样式

### Requirement: 动效克制与可访问性

landing 组件的动效 MUST 仅使用 CSS transition/animation 且只作用于 `transform` 与 `opacity`，并 MUST 在 `prefers-reduced-motion: reduce` 下全部塌缩为静态。

#### Scenario: 减少动效偏好

- **WHEN** 以 `prefers-reduced-motion: reduce` 模拟打开首页
- **THEN** 页面无自动播放的动画，所有内容直接以最终状态呈现
