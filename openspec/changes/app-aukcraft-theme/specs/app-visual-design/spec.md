# app-visual-design 能力规格（delta）

## ADDED Requirements

### Requirement: 中性色核心 token

设置窗口主题 SHALL 原样落地 aukcraft 中性色核心：窗口背景 `#0B0E11`、浮层表面 `#14181D`、边框/分隔线为 8% 白色发丝线（或其不透明近似值）、主文字 `#EDEDED`、次要文字 `#8A9199`。主题 MUST 固定为暗色，MUST NOT 使用纯 `#000` / `#FFF`，MUST NOT 提供亮色模式。

#### Scenario: 主题变量落地

- **WHEN** 查看 `src/index.css` 的 `:root` 变量块
- **THEN** `--background` / `--card` / `--popover` / `--foreground` / `--muted-foreground` 的值与中性色核心一致（HSL 等价），且不存在 `.light` 或媒体查询亮色变体

#### Scenario: 卡片浮于窗口之上

- **WHEN** 设置窗口渲染任意 Card / Popover
- **THEN** 浮层表面颜色（`#14181D`）与窗口背景（`#0B0E11`）有可辨明度差，层次由明度差 + 1px 发丝线表达

### Requirement: accent 配给纪律

设置窗口 MUST 只声明一个产品 accent（品牌蓝 `#2563EB` 家族），暗色界面上 MUST 使用 400 阶 `#60A5FA` 以保证与窗口背景对比度 ≥ 4.5:1。accent MUST 只出现在：焦点指示、每窗口唯一主操作、链接、选中态标记、实时状态。accent MUST NOT 用于背景填充、徽标、装饰线、默认图标；shadcn 的 `--accent` 悬停底色槽位 MUST 保持中性。

#### Scenario: 主按钮使用 accent 而非白色

- **WHEN** 查看主操作按钮（`bg-primary`）
- **THEN** `--primary` 为 accent 400 阶（`#60A5FA`），不是白色或近白色

#### Scenario: 焦点环为 1px accent 描边

- **WHEN** 键盘 Tab 聚焦任意可交互控件
- **THEN** 焦点指示为 1px accent 描边 + 3px 偏移（`ring-1 ring-ring ring-offset-[3px]`），不是 shadcn 默认的粗光晕

#### Scenario: 悬停底色不放品牌色

- **WHEN** 悬停在列表项 / 幽灵按钮 / 下拉项上
- **THEN** 悬停底色为中性表面（`--accent` 与 `--muted` 同值的 raised 色），不出现品牌蓝填充

### Requirement: 圆角与阴影纪律

app 自有界面 MUST 零阴影，圆角 MUST ≤ 4px（`--radius: 4px`，md 3px，sm 2px）。层次 MUST 只由明度差 + 1px 发丝线表达。OS 原生菜单、托盘、系统工具提示豁免。

#### Scenario: 基础组件无阴影

- **WHEN** 检查 `src/components/ui/` 与功能组件的 class
- **THEN** 不存在任何 `shadow-*` 工具类（弹层靠 raised 表面 + 发丝线 + 遮罩分层）

#### Scenario: 圆角上限

- **WHEN** 检查 `--radius` 及各组件圆角用法
- **THEN** `--radius` 为 4px，除功能性圆形元素（单选框、开关、滑块 Thumb）外无超过 4px 的圆角

### Requirement: 动效纪律

所有交互过渡 MUST 使用 `ease-lock` 缓动（`cubic-bezier(0.16, 1, 0.3, 1)`）。MUST NOT 存在环境动效、待机动效、循环动效——无人触碰的窗口是一块完全静止的表面。`prefers-reduced-motion` 下 MUST 降级为直接变色、禁用缩放。

#### Scenario: 过渡缓动统一

- **WHEN** 检查 `src/index.css` 与组件过渡类
- **THEN** 交互元素过渡统一为 `ease-lock` 缓动

#### Scenario: reduced-motion 降级

- **WHEN** 系统开启「减少动态效果」
- **THEN** 动画与过渡时长归零，交互状态变化以直接变色表达

### Requirement: 升级边界（保留清单）

升级 MUST 只改视觉层（token + 基础组件 class），MUST NOT 重组布局、tab、交互流程或更换组件库。以下既有决策 MUST 保留：自动隐藏滚动条、紧凑信息密度（`p-6` / `space-y-6` 级）、固定暗色主题、运行时 i18n、平台原生组件。

#### Scenario: diff 只含 token/class 级改动

- **WHEN** 审查本变更的代码 diff
- **THEN** 改动集中于 `src/index.css` 与 `src/components/ui/*`，功能组件除删除 `shadow-*` 外零改动，无布局结构变化

#### Scenario: 保留项未被破坏

- **WHEN** 升级完成后走查设置窗口
- **THEN** 滚动条仍为悬停淡入的 6px 细轨道、内容密度未膨胀、i18n 运行时切换正常、原生标题栏/托盘未改
