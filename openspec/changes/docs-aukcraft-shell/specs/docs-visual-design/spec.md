## ADDED Requirements

### Requirement: aukcraft 壳层表面语言

文档站全站（落地页 + guide 页）MUST 采用 aukcraft 壳层的表面语言：零阴影、圆角 ≤4px（容器 4px / 行内 2px）、hairline 边框分层；明暗双主题 MUST 分别为「蓝调近黑」与「蓝调近白」（非纯黑/纯白）。

#### Scenario: 零阴影

- **WHEN** 检查任一落地页或 guide 页面的卡片、截图框、代码块等容器元素
- **THEN** 无 `box-shadow` 阴影，分层仅由表面亮度差与 1px hairline 边框表达

#### Scenario: 圆角收紧

- **WHEN** 检查落地页按钮、代码块、aside、表格、截图框等容器
- **THEN** 圆角不超过 4px，不存在 pill（`border-radius: 999px`）按钮

#### Scenario: 双主题非纯黑纯白

- **WHEN** 分别以明暗两种主题打开首页
- **THEN** 暗色页面底色为蓝调近黑（`#0B0E11`）、浅色页面底色为蓝调近白（`#F5F7F9`），均非 `#000` 或 `#FFF`

#### Scenario: 胶片颗粒范围

- **WHEN** 对比首页与任一 guide 页 / download 页
- **THEN** 仅首页存在 `.noise` 胶片颗粒 overlay（fixed、pointer-events-none、透明度 ≤3%），guide 页与 download 页正文无该层

### Requirement: 落地页签名动效组件

落地页 MUST 提供 aukcraft 签名动效：`DotField` 点阵动态背景（光标附近点阵填蓝，距离衰减）与 `FlightLine` 边框描边按钮；两者 MUST 在 `prefers-reduced-motion: reduce` 下塌缩为静态（点阵只画静态空心环、描边瞬时常亮）。

#### Scenario: DotField 点阵背景

- **WHEN** 打开 `/` 或 `/zh-cn/` 首页
- **THEN** 页面内容后方呈现点阵背景，光标移动时附近点阵以蓝色渐变填充

#### Scenario: FlightLine 描边按钮

- **WHEN** 悬停或聚焦落地页的下载 CTA 按钮
- **THEN** 按钮边框出现沿边框描边一周的动效，结束后整圈常亮直至取消悬停/失焦

#### Scenario: 减少动效偏好

- **WHEN** 以 `prefers-reduced-motion: reduce` 打开首页
- **THEN** 点阵不随光标填充、描边不播放动画，仅以静态最终状态呈现

## MODIFIED Requirements

### Requirement: 品牌设计 token 单一事实源

品牌蓝 `#2563EB` 与 aukcraft 蓝调中性灰阶 MUST 在 Tailwind `@theme` 中以 `--color-accent-*` 与 `--color-gray-*` 单处定义（Starlight 原生消费这两组 token），不得在其他 CSS 文件中手写相同色值；中性灰阶两端 MUST 锚定暗色 base `#0B0E11` / raised `#14181D`、浅色 base `#F5F7F9`；品牌蓝 MUST 仅用于链接、CTA、描边动线、焦点环与画布光标填色，不用于大面积背景或装饰。

#### Scenario: 色值单源

- **WHEN** 检查 `src/styles/` 下品牌蓝与中性灰阶色值的定义位置
- **THEN** 十六进制色值仅出现在 `@theme` 块内，其余文件通过 `var()` 或工具类引用

#### Scenario: 蓝调中性锚点

- **WHEN** 检查 `@theme` 中的 `--color-gray-*` 取值
- **THEN** `--color-gray-950` 为 `#0B0E11`、`--color-gray-900` 为 `#14181D`、`--color-gray-50` 为 `#F5F7F9`

#### Scenario: 主色节制

- **WHEN** 浏览首页与任一 guide 页面
- **THEN** 品牌蓝只出现在链接、CTA、描边动线、焦点环与光标填色上，无蓝色大面积背景或装饰性铺色

#### Scenario: 暗色对比度兜底

- **WHEN** 以暗色主题检查落地页 CTA 按钮与链接文字
- **THEN** 文字与背景对比度 ≥ 4.5:1（允许使用 `--color-accent-400` 等亮阶承载暗色下的蓝色文字与描边动线）

### Requirement: 首页落地页构成

首页（en 与 zh-cn）MUST 呈现 aukcraft 壳层落地页：自定义 Hero（左文右图，真实产品截图 + serif-italic 标题强调 + 零阴影 hairline 截图框）、DotField 点阵背景、特性网格、三步上手区块、下载入口区块（FlightLine 描边按钮）；且 MUST 保持 `template: splash` 与 Starlight 顶栏（语言切换、主题开关、GitHub 链接）可用。

#### Scenario: Hero 结构

- **WHEN** 打开 `/` 或 `/zh-cn/`
- **THEN** Hero 左侧为标题（含 serif-italic 关键词强调）、不超过 20 词的副标题与至多两个 CTA，右侧为真实产品截图（零阴影 hairline 边框），首屏内 CTA 可见无需滚动

#### Scenario: 落地页区块完整

- **WHEN** 滚动首页
- **THEN** 依次出现特性网格、三步上手、下载入口三个区块，且各区块布局模式不重复

#### Scenario: 双语一致

- **WHEN** 对比 `/` 与 `/zh-cn/` 首页
- **THEN** 两者区块结构、组件构成完全一致，仅文案语言不同

#### Scenario: Starlight 顶栏保留

- **WHEN** 在首页点击语言切换或主题开关
- **THEN** 正常跳转对应 locale 或切换明暗主题，功能不受 Hero 覆写影响

### Requirement: 文档页排版精修

guide 页面 MUST 在不新增 Starlight 组件覆写的前提下（仅通过 `customCss`），完成配色层次与排版细节校准，覆盖正文、code block、aside、表格、链接；这些内容区元素 MUST 统一为零阴影 + ≤4px 圆角 + hairline 边框分层，且明暗双主题各自完整。侧栏 pill、搜索框、主题切换等 Starlight 自带控件 MAY 在 `customCss` 层定点覆写 radius/shadow 以纳入同一表面语言，但 MUST NOT 为此新增 `components:` 覆写。

#### Scenario: 精修不覆写组件

- **WHEN** 检查 `astro.config.mjs` 的 `components` 配置
- **THEN** 除 `Hero` 与 `Header` 外无其他 Starlight 组件覆写

#### Scenario: 双主题观感

- **WHEN** 在明暗两种主题下打开任一 guide 页面
- **THEN** 两种主题下正文对比度、code block、aside、表格样式均完整可读，无未适配的裸默认样式

#### Scenario: 内容区表面语言

- **WHEN** 检查 guide 页面的 code block、aside、表格
- **THEN** 均无阴影、圆角 ≤4px、以 hairline 边框分层

#### Scenario: 控件 CSS 级换皮

- **WHEN** 检查侧栏导航 pill、搜索框、主题切换控件的样式来源
- **THEN** 其 ≤4px 圆角与零阴影由 `customCss` 定点覆写实现，且每条覆写规则带有目标 Starlight 版本与内部 class 的注释

### Requirement: 动效克制与可访问性

landing 组件的动效 MUST 使用 CSS transition/animation（仅作用于 `transform` 与 `opacity`）、原生 canvas 光标填色或 SVG 描边动效实现，且 MUST 在 `prefers-reduced-motion: reduce` 下全部塌缩为静态；不得引入自动播放或循环动画（不引入点阵字标 idle wave 类循环动效）。

#### Scenario: 减少动效偏好

- **WHEN** 以 `prefers-reduced-motion: reduce` 模拟打开首页
- **THEN** 页面无自动播放或循环的动画，所有内容（含点阵、描边、reveal）直接以最终状态呈现

#### Scenario: 无循环动画

- **WHEN** 停留首页观察
- **THEN** 除光标触发的点阵填充与描边动效外，无持续自动循环的动画
