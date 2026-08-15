# 提案：设置窗口样式升级为 aukcraft 桌面应用设计规范

> **跟踪 issue：#76**（https://github.com/Eeymoo/peregrine/issues/76）

## Why

设置窗口目前使用 shadcn/ui 默认锌灰暗色主题：窗口与卡片同为 `240 10% 3.9%` 灰黑（卡片浮不起来）、白色主按钮（`--primary: 0 0% 98%`）是明显的品牌断裂点、组件库自带阴影与 0.5rem 药丸圆角与 aukcraft 家族视觉语言不一致。aukcraft-app-design 技能已落地（`.agents/skills/aukcraft-app-design/`），其中 `references/peregrine-mapping.md` 给出了本项目专用的「现状 → 目标」映射表，应按其执行升级，让 app 与产品站点读作一家人。

## 目标

- 中性色核心原样落地：`#0B0E11` 窗口底 / `#14181D` 浮层 / 8% 白发丝线 / `#EDEDED` 主文字 / `#8A9199` 次要文字
- accent 换为品牌蓝 `#2563EB` 家族，暗色界面统一用 400 阶 `#60A5FA`（≥4.5:1 对比度）
- 圆角 ≤4px、app 自有界面零阴影、焦点环 1px accent 描边 + 3px 偏移
- 交互过渡统一 `ease-lock` 缓动 + `prefers-reduced-motion` 降级
- 布局、tab、交互流程、组件库零改动——diff 只含 token/class 级改动

## 非目标

- 不重组布局、不更换组件库、不引入亮色主题
- 不移植网站的动态背景 / 玻璃拟态 / 板块节奏 / 衬线点缀
- 不改任何 Rust 后端逻辑与配置 schema
- 不做全量 micro 标签改造（仅提供 `.micro-label` 工具类，后续按需启用）

## What Changes

- `src/index.css`：替换 shadcn 暗色变量块为 aukcraft 主题（由 `generate_theme.py --accent "#2563EB" --accent-soft "#60A5FA" --format shadcn` 生成），`--radius` 收到 4px，新增 ease-lock 全局过渡、reduced-motion 降级、`.micro-label` 工具类、aukcraft 原始 token 变量
- `src/components/ui/*`（button/card/tabs/checkbox/radio-group/select/switch/slider/kbd）：剥除 `shadow-*`，焦点环从 `ring-2 ring-offset-2` 改为 1px accent 描边 + 3px 偏移
- 6 处功能组件弹层/手写按钮（UpdateProgress、UpdateDialog、AutoSwitchDialog、LayerPanel、LayersEditor、ConfigApp）：仅删除 `shadow-*` 类，其余零改动

## Capabilities

### New Capabilities

- `app-visual-design`：桌面设置窗口的视觉规范约束（中性色核心 token、accent 配给纪律、圆角/阴影/焦点环/动效纪律、保留清单）

### Modified Capabilities

（无——本变更新增视觉规范能力，不修改既有能力的需求）

## Impact

- 受影响代码：`src/index.css`、`src/components/ui/`（9 个基础组件）、`src/ConfigApp.tsx`、`src/components/{LayerPanel,LayersEditor}.tsx`、`src/components/config/{UpdateProgress,UpdateDialog,AutoSwitchDialog}.tsx`
- 不影响：Rust 后端、配置 schema、构建流程、i18n key
- 验证方式：`npm run build`（tsc 类型检查 + Vite 打包）+ 960×560 真实窗口人工走查
