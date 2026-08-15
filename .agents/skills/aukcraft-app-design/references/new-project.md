# 新建项目

从零开始一个 aukcraft 家族的桌面 app。没有历史包袱，核心思路是：**从中性色核心直接生长，第一天就按规则实现，不留"以后统一"的债**。

## 流程

### 1. 落地中性色核心 + 声明产品 accent

先用主题生成脚本产出主题骨架，作为项目的第一个样式文件：

```bash
# 选定产品 accent（与产品站点一致），生成对应格式的主题
python scripts/generate_theme.py --accent "<产品accent>" --format css      # 通用
python scripts/generate_theme.py --accent "<产品accent>" --format shadcn   # shadcn/ui 项目
python scripts/generate_theme.py --accent "<产品accent>" --format tailwind # Tailwind config 片段
```

一个 app 只声明**一个** accent。拿不准时用组织级的 Auk Teal `#14B8A6`；产品已有站点时与站点对齐（如 Peregrine 的品牌蓝）。

### 2. 按工具形态选布局语法

布局一旦选定就贯彻到底，不要在一个 app 里混用多种：

| 工具形态 | 布局语法 | 例子 |
|---|---|---|
| 多分组设置 | tabs + cards | Peregrine 设置窗口（general / overlay / hotkeys / about） |
| 主从结构 | 侧栏列表 + 内容区 | 多配置档案管理器 |
| 单一任务 | 单窗口表单 + 底部主操作 | 一次性配置向导、授权对话框 |
| 常驻辅助 | 托盘 + 按需唤出的紧凑面板 | 状态监控、快捷开关 |

通用结构规则：

- 窗口背景 `base`；分组/卡片 `raised` + 发丝线边框；模态层 `raised` + 发丝线，压在 60–70% 黑色遮罩上
- 分组之间靠**一条发丝线 + 一个 micro 标签**分隔，不靠大留白
- 每个窗口/对话框只有**一个** accent 主操作，通常放在底部操作区

### 3. 控件按规范实现

- 高度阶梯：28px 紧凑行 / 32px 默认 / 36px 主操作；8px 间距网格
- 焦点：每个可交互元素有 1px accent 描边（3px 偏移）的可见焦点态；完整键盘可达；Escape 关闭对话框
- 圆角 ≤ 4px；零阴影；边框一律 1px 发丝线
- 过渡统一 `ease-lock`；`:active` 缩放 0.98；`prefers-reduced-motion` 降级从第一版就写上，不后补
- 分组标题/状态用 micro 标签（JetBrains Mono、全大写、`0.15em` 字距、`muted`）

### 4. 平台事项

- 优先使用**原生标题栏**，除非产品确实需要自定义（如无框悬浮层配置器）。自定义标题栏用 `raised` + 下方一条发丝线，窗口控制按钮服从平台惯习（Windows 右侧最小化/最大化/关闭）
- 托盘/菜单栏完全遵循平台惯习——原生菜单、原生长相；品牌规则只适用于 app 自己的窗口
- i18n 用运行时切换；关键技术术语（crate、CI、PR、TDD）在所有语言中保持英文

## 新建情形检查清单

在 SKILL.md 共用清单之外，新建情形额外确认：

- [ ] 主题由生成脚本产出或与其逐值一致，无手写的近似色
- [ ] 只声明了一个产品 accent，且与产品站点对齐
- [ ] 布局语法单一，全 app 贯彻（无 tabs 与侧栏混用）
- [ ] 分组分隔统一为发丝线 + micro 标签
- [ ] reduced-motion 降级、键盘可达、Escape 语义从第一版就位
